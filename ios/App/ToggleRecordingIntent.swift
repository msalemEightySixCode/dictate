import AppIntents

/// LiveActivityIntent runs this in the app process without opening the app.
struct ToggleRecordingIntent: AudioRecordingIntent, LiveActivityIntent {
    static let title: LocalizedStringResource = "Start or Stop Dictation"
    static let openAppWhenRun = false

    @MainActor
    func perform() async throws -> some IntentResult {
        try await Recorder.shared.toggle(source: "Action Button")
        return .result()
    }
}

struct DictateShortcuts: AppShortcutsProvider {
    static var appShortcuts: [AppShortcut] {
        AppShortcut(
            intent: ToggleRecordingIntent(),
            phrases: ["Dictate with \(.applicationName)"],
            shortTitle: "Dictate",
            systemImageName: "waveform"
        )
    }
}
