import AVFoundation
import SwiftUI

@main
struct HandyDictateApp: App {
    var body: some Scene {
        WindowGroup { ContentView() }
    }
}

struct ContentView: View {
    @ObservedObject private var recorder = Recorder.shared
    @State private var player: AVAudioPlayer?
    @State private var micAllowed = AVAudioApplication.shared.recordPermission == .granted
    @State private var problem: String?

    var body: some View {
        List {
            Section("Setup") {
                if micAllowed {
                    Label("Microphone allowed", systemImage: "checkmark")
                } else {
                    Button("Allow microphone") {
                        Task { micAllowed = await AVAudioApplication.requestRecordPermission() }
                    }
                }
            }
            Section("Test") {
                Button(recorder.isRecording ? "Stop" : "Start from this screen") {
                    Task {
                        do { try await recorder.toggle(source: "Screen") }
                        catch { problem = error.localizedDescription }
                    }
                }
                Button("Play last recording (\(Recorder.savedBytes) bytes)") {
                    try? AVAudioSession.sharedInstance().setCategory(.playback)
                    try? AVAudioSession.sharedInstance().setActive(true)
                    player = try? AVAudioPlayer(contentsOf: Recorder.fileURL)
                    player?.play()
                }
                .disabled(recorder.isRecording || Recorder.savedBytes == 0)
                if let problem { Text(problem).foregroundStyle(.red) }
            }
            Section("Log") {
                ForEach(recorder.log, id: \.self) { Text($0).font(.footnote.monospaced()) }
            }
        }
        .task { await recorder.endStaleActivities() }
    }
}
