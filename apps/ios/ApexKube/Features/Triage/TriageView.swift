import SwiftUI
import Triage

/// Placeholder triage home. Real data arrives in PR4.
struct TriageView: View {
    private let placeholder = Verdict(status: .unknown, reason: "No cluster connected", ruleId: "TRI-NONE")

    var body: some View {
        NavigationStack {
            VStack(spacing: 12) {
                Text("Apex")
                    .font(.largeTitle.bold())
                Text("Triage")
                    .font(.title3)
                    .foregroundStyle(.secondary)
                Label(placeholder.reason, systemImage: "circle.dotted")
                    .foregroundStyle(.secondary)
            }
            .padding()
            .navigationTitle("Triage")
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}

#Preview {
    TriageView()
}
