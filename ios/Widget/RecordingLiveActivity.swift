import ActivityKit
import SwiftUI
import WidgetKit

@main
struct DictateWidgets: WidgetBundle {
    var body: some Widget { RecordingLiveActivity() }
}

struct RecordingLiveActivity: Widget {
    var body: some WidgetConfiguration {
        ActivityConfiguration(for: RecordingAttributes.self) { context in
            HStack {
                Image(systemName: "waveform").foregroundStyle(.white)
                Text("Recording").foregroundStyle(.white)
                Spacer()
                timer(context.state.startedAt)
            }
            .padding()
            .activityBackgroundTint(.black)
            .activitySystemActionForegroundColor(.white)
        } dynamicIsland: { context in
            DynamicIsland {
                DynamicIslandExpandedRegion(.leading) {
                    Image(systemName: "waveform").foregroundStyle(.white)
                }
                DynamicIslandExpandedRegion(.trailing) {
                    timer(context.state.startedAt)
                }
                DynamicIslandExpandedRegion(.bottom) {
                    Text("Press the Action Button to stop").font(.caption).foregroundStyle(.white)
                }
            } compactLeading: {
                Image(systemName: "waveform").foregroundStyle(.white)
            } compactTrailing: {
                timer(context.state.startedAt).frame(width: 40)
            } minimal: {
                Image(systemName: "waveform").foregroundStyle(.white)
            }
            .keylineTint(.white)
        }
    }

    private func timer(_ start: Date) -> some View {
        Text(start, style: .timer)
            .monospacedDigit()
            .multilineTextAlignment(.trailing)
            .foregroundStyle(.white)
    }
}
