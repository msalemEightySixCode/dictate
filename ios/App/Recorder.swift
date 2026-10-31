import ActivityKit
import AVFoundation
import Foundation

/// Recording state lives here so the Action Button intent and the screen share it.
@MainActor
final class Recorder: ObservableObject {
    static let shared = Recorder()

    @Published private(set) var isRecording = false
    @Published private(set) var log: [String] = UserDefaults.standard.stringArray(forKey: "log") ?? []

    private var recorder: AVAudioRecorder?
    private var activity: Activity<RecordingAttributes>?

    static var fileURL: URL {
        FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("last.m4a")
    }

    func toggle(source: String) async throws {
        if isRecording { await stop(source: source) } else { try await start(source: source) }
    }

    private func start(source: String) async throws {
        write("\(source): start")
        guard AVAudioApplication.shared.recordPermission == .granted else {
            write("\(source): no microphone permission")
            throw DictateError.noMicrophonePermission
        }
        do {
            // Apple ends the recording if no Live Activity is open, so it starts first.
            activity = try Activity.request(
                attributes: RecordingAttributes(),
                content: .init(state: .init(startedAt: Date()), staleDate: nil)
            )
            write("\(source): bar started")

            let session = AVAudioSession.sharedInstance()
            try session.setCategory(.record, mode: .default)
            try session.setActive(true)

            let new = try AVAudioRecorder(url: Self.fileURL, settings: [
                AVFormatIDKey: kAudioFormatMPEG4AAC,
                AVSampleRateKey: 16_000,
                AVNumberOfChannelsKey: 1,
            ])
            guard new.record() else { throw DictateError.recordFailed }
            recorder = new
            isRecording = true
            write("\(source): recording")
        } catch {
            write("\(source): FAILED \(error.localizedDescription)")
            await endActivity()
            throw error
        }
    }

    private func stop(source: String) async {
        recorder?.stop()
        recorder = nil
        isRecording = false
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
        await endActivity()
        write("\(source): saved \(Self.savedBytes) bytes")
    }

    private func endActivity() async {
        await activity?.end(nil, dismissalPolicy: .immediate)
        activity = nil
    }

    /// A killed app leaves its bar open, and nothing can be recording at launch.
    func endStaleActivities() async {
        guard !isRecording else { return }
        for old in Activity<RecordingAttributes>.activities {
            await old.end(nil, dismissalPolicy: .immediate)
        }
    }

    static var savedBytes: Int {
        (try? FileManager.default.attributesOfItem(atPath: fileURL.path)[.size] as? Int) ?? 0
    }

    private func write(_ line: String) {
        let time = Date().formatted(date: .omitted, time: .standard)
        log.insert("\(time) \(line)", at: 0)
        log = Array(log.prefix(40))
        UserDefaults.standard.set(log, forKey: "log")
    }
}

enum DictateError: LocalizedError {
    case noMicrophonePermission, recordFailed
    var errorDescription: String? {
        switch self {
        case .noMicrophonePermission: "Open Dictate and allow the microphone."
        case .recordFailed: "The recording did not start."
        }
    }
}
