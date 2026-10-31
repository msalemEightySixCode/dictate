<p align="center">
  <img src="docs/assets/banner.png" alt="Dictate: speech to text. Local. Free." width="100%">
</p>

<p align="center">
  <b>Private speech to text for your Mac, with your iPhone as a second microphone.</b><br>
  My personal take on <a href="https://github.com/cjpais/Handy">Handy</a> by CJ Pais.
</p>

<p align="center">
  <img alt="macOS" src="https://img.shields.io/badge/macOS-000000?style=flat-square&logo=apple&logoColor=white">
  <img alt="Runs offline" src="https://img.shields.io/badge/transcription-on_your_Mac-000000?style=flat-square">
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-v2-000000?style=flat-square&logo=tauri&logoColor=white">
  <img alt="MIT license" src="https://img.shields.io/badge/license-MIT-000000?style=flat-square">
</p>

---

## What it does

Press your shortcut and speak. Dictate transcribes your voice on your own Mac and pastes the text at your cursor. Your audio does not go to a cloud service.

## What is different from Handy

| | Handy | Dictate |
|---|---|---|
| **Name and logo** | Handy hand and pink wordmark | Dictate name and signal logo |
| **Colors** | Pink accent | Black and white only, with white buttons and black text |
| **Recording overlay** | Pink pill with a dot and a cancel button | Small black and white pill with sound bars only |
| **Sound bars** | Follow the frequency levels | Follow your loudness above the noise in the room |
| **iPhone** | Not included | Send audio from your iPhone to your Mac over Tailscale |
| **Status** | Not included | The footer shows if the iPhone server is on |

Press <kbd>Esc</kbd> to cancel a recording, because the overlay has no cancel button.

## Use your iPhone

The Mac runs a small server on port `8765`. It accepts only your own Mac and devices on your Tailscale network, and every request needs your private token.

1. Install [Tailscale](https://tailscale.com) on your Mac and iPhone.
2. Open Dictate on your Mac, and check that the footer shows **iPhone server on**.
3. Hold the pointer over the status to see your Tailscale address.
4. Copy your token from `~/Library/Application Support/com.pais.handy/phone_server_token`.
5. On your iPhone, create a Shortcut with these three actions:

| Action | Setting |
|---|---|
| **Record Audio** | Start immediately, finish on tap |
| **Get Contents of URL** | `http://<your-tailscale-address>:8765/transcribe`, method POST, header `Authorization: Bearer <your-token>`, request body File with the recorded audio |
| **Copy to Clipboard** | The contents of the URL |

6. Set the Shortcut on your Action Button.

> [!WARNING]
> **Work in progress:** the [`ios`](ios) folder holds a native iPhone app with a Dynamic Island bar. It has not been compiled or run on an iPhone yet. iOS allows text at the cursor only through a custom keyboard, and that part is not built.

## Build it

You need [Rust](https://rustup.rs) and [Bun](https://bun.sh). See [BUILD.md](BUILD.md) for the full requirements.

```bash
bun install --frozen-lockfile
bun run tauri build --bundles app --config '{"bundle":{"createUpdaterArtifacts":false}}'
```

The app is in `src-tauri/target/release/bundle/macos/`. It is not notarized, so you must approve it in macOS yourself. You must also add it to **Privacy & Security > Accessibility** again after each new build.

I tested this version only on macOS. The upstream Windows and Linux code is still here, but I have not tested it with these changes.

## Credit

Dictate is built on **[Handy](https://github.com/cjpais/Handy)**, created by **[CJ Pais](https://github.com/cjpais)** and its contributors. Handy does all of the hard work: the recording, the models, the shortcuts and the paste. The full Handy commit history is kept in this repository.

Please support the original project at [handy.computer](https://handy.computer). Dictate is an independent personal project and is not affiliated with Handy. Do not send bugs from Dictate to the Handy team.

## License

[MIT](LICENSE). The original copyright belongs to CJ Pais, and it stays in the license file.
