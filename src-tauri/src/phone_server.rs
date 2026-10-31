//! Local mod: lets an iPhone Shortcut send audio to this Mac over Tailscale and
//! get the text back, using the model Handy already holds.
//! POST /transcribe, `Authorization: Bearer <token>`, raw audio file as the body.

use crate::managers::transcription::TranscriptionManager;
use log::{info, warn};
use serde::Serialize;
use specta::Type;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream, UdpSocket};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

const PORT: u16 = 8765;
const MAX_HEAD: usize = 16 * 1024;
const MAX_BODY: usize = 50 * 1024 * 1024;

pub struct PhoneServerState {
    listening: AtomicBool,
}

#[derive(Serialize, Type)]
pub struct PhoneServerStatus {
    listening: bool,
    tailscale_ip: Option<String>,
    port: u16,
}

#[tauri::command]
#[specta::specta]
pub fn get_phone_server_status(state: State<Arc<PhoneServerState>>) -> PhoneServerStatus {
    PhoneServerStatus {
        listening: state.listening.load(Ordering::Relaxed),
        tailscale_ip: tailscale_ip(),
        port: PORT,
    }
}

pub fn start(app: &AppHandle, manager: Arc<TranscriptionManager>) {
    let state = Arc::new(PhoneServerState {
        listening: AtomicBool::new(false),
    });
    app.manage(state.clone());

    let token = match load_token(app) {
        Ok(token) => token,
        Err(e) => {
            warn!("Phone server off, no token: {e}");
            return;
        }
    };
    // All interfaces, because Tailscale can start after Handy. Every peer outside
    // Tailscale is refused below.
    let listener = match TcpListener::bind(("0.0.0.0", PORT)) {
        Ok(listener) => listener,
        Err(e) => {
            warn!("Phone server off, cannot bind port {PORT}: {e}");
            return;
        }
    };
    state.listening.store(true, Ordering::Relaxed);
    info!("Phone server listening on port {PORT}");

    std::thread::spawn(move || {
        // shortcut: one request at a time, which is enough for one iPhone.
        for stream in listener.incoming().flatten() {
            if let Err(e) = handle(stream, &token, &manager) {
                warn!("Phone server request failed: {e}");
            }
        }
    });
}

/// The address the OS would use to reach Tailscale's DNS. Sends no packets.
fn tailscale_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("100.100.100.100:53").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    is_tailscale(ip).then(|| ip.to_string())
}

fn is_tailscale(ip: IpAddr) -> bool {
    matches!(ip, IpAddr::V4(v4) if v4.octets()[0] == 100 && (64..128).contains(&v4.octets()[1]))
}

fn allowed_peer(ip: IpAddr) -> bool {
    ip.is_loopback() || is_tailscale(ip)
}

fn load_token(app: &AppHandle) -> anyhow::Result<String> {
    let path = crate::portable::app_data_dir(app)?.join("phone_server_token");
    if let Ok(token) = std::fs::read_to_string(&path) {
        let token = token.trim().to_string();
        if token.len() >= 32 {
            return Ok(token);
        }
    }
    let mut bytes = [0u8; 24];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)?;
    file.write_all(token.as_bytes())?;
    Ok(token)
}

#[derive(Debug, PartialEq)]
struct Head {
    method: String,
    path: String,
    token: Option<String>,
    content_length: Option<usize>,
    expect_continue: bool,
}

fn parse_head(head: &str) -> Option<Head> {
    let mut lines = head.split("\r\n");
    let mut request = lines.next()?.split(' ');
    let (method, path) = (request.next()?.to_string(), request.next()?.to_string());
    let (mut token, mut content_length, mut expect_continue) = (None, None, false);
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        // trim: iOS Shortcuts can send a key with a trailing space.
        match name.trim().to_ascii_lowercase().as_str() {
            "authorization" => token = value.strip_prefix("Bearer ").map(str::to_string),
            "content-length" => content_length = Some(value.parse().ok()?),
            "expect" => expect_continue = value.eq_ignore_ascii_case("100-continue"),
            _ => {}
        }
    }
    Some(Head {
        method,
        path,
        token,
        content_length,
        expect_continue,
    })
}

fn token_matches(given: &str, token: &str) -> bool {
    given.len() == token.len()
        && given
            .bytes()
            .zip(token.bytes())
            .fold(0u8, |diff, (a, b)| diff | (a ^ b))
            == 0
}

fn respond(stream: &mut TcpStream, status: &str, body: &str) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn handle(
    mut stream: TcpStream,
    token: &str,
    manager: &TranscriptionManager,
) -> anyhow::Result<()> {
    if !allowed_peer(stream.peer_addr()?.ip()) {
        return Ok(()); // Drop silently: a stranger on the LAN learns nothing.
    }
    stream.set_read_timeout(Some(Duration::from_secs(60)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut data = Vec::new();
    let mut chunk = [0u8; 8192];
    let head_end = loop {
        if let Some(i) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        if data.len() > MAX_HEAD {
            return Ok(respond(&mut stream, "431 Too Large", "Dictate error: bad request")?);
        }
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Ok(());
        }
        data.extend_from_slice(&chunk[..n]);
    };
    let Some(head) = std::str::from_utf8(&data[..head_end]).ok().and_then(parse_head) else {
        return Ok(respond(&mut stream, "400 Bad Request", "Dictate error: bad request")?);
    };

    if head.method != "POST" || head.path != "/transcribe" {
        return Ok(respond(&mut stream, "404 Not Found", "Dictate error: not found")?);
    }
    if !head.token.as_deref().is_some_and(|t| token_matches(t, token)) {
        warn!(
            "Phone server refused token: header present={}, length={:?}",
            head.token.is_some(),
            head.token.as_ref().map(String::len)
        );
        return Ok(respond(&mut stream, "401 Unauthorized", "Dictate error: wrong token")?);
    }
    let Some(length) = head.content_length else {
        return Ok(respond(&mut stream, "411 Length Required", "Dictate error: no length")?);
    };
    if length > MAX_BODY {
        return Ok(respond(&mut stream, "413 Too Large", "Dictate error: audio too long")?);
    }
    if head.expect_continue {
        stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n")?;
    }

    let mut body = data.split_off(head_end + 4);
    while body.len() < length {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Ok(());
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(length);

    match transcribe(&body, manager) {
        Ok(text) => respond(&mut stream, "200 OK", &text)?,
        Err(e) => {
            warn!("Phone server transcription failed: {e}");
            respond(&mut stream, "500 Error", "Dictate error: could not transcribe")?;
        }
    }
    Ok(())
}

fn transcribe(audio: &[u8], manager: &TranscriptionManager) -> anyhow::Result<String> {
    let (input, output) = (temp_path("in"), temp_path("out.wav"));
    let result = (|| {
        std::fs::write(&input, audio)?;
        // afconvert ships with macOS and reads what iOS records (m4a, caf, wav).
        let converted = Command::new("/usr/bin/afconvert")
            .args(["-f", "WAVE", "-d", "LEI16@16000", "-c", "1", "--mix"])
            .arg(&input)
            .arg(&output)
            .output()?;
        if !converted.status.success() {
            anyhow::bail!("afconvert: {}", String::from_utf8_lossy(&converted.stderr));
        }
        let samples = crate::audio_toolkit::read_wav_samples(&output)?;
        manager.initiate_model_load(); // The model unloads when idle.
        manager.transcribe(samples)
    })();
    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&output);
    result
}

// temp_dir is private to this user on macOS, and requests run one at a time.
fn temp_path(suffix: &str) -> PathBuf {
    std::env::temp_dir().join(format!("handy-phone-{}-{suffix}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_tailscale_and_loopback_may_connect() {
        for (ip, ok) in [
            ("100.64.0.1", true),
            ("100.127.255.254", true),
            ("127.0.0.1", true),
            ("100.63.255.255", false),
            ("100.128.0.1", false),
            ("192.168.1.20", false),
            ("10.0.0.2", false),
        ] {
            assert_eq!(allowed_peer(ip.parse().unwrap()), ok, "{ip}");
        }
    }

    #[test]
    fn reads_the_request_head() {
        let head = parse_head(
            "POST /transcribe HTTP/1.1\r\nauthorization: Bearer abc\r\nCONTENT-LENGTH: 12\r\nExpect: 100-continue",
        )
        .unwrap();
        assert_eq!(head.method, "POST");
        assert_eq!(head.path, "/transcribe");
        assert_eq!(head.token.as_deref(), Some("abc"));
        assert_eq!(head.content_length, Some(12));
        assert!(head.expect_continue);
        assert!(parse_head("POST /transcribe HTTP/1.1\r\nContent-Length: x").is_none());
        let spaced = parse_head("POST / HTTP/1.1\r\nAuthorization : Bearer abc").unwrap();
        assert_eq!(spaced.token.as_deref(), Some("abc"));
    }

    #[test]
    fn token_must_match_exactly() {
        assert!(token_matches("abc", "abc"));
        assert!(!token_matches("abd", "abc"));
        assert!(!token_matches("ab", "abc"));
        assert!(!token_matches("", "abc"));
    }
}
