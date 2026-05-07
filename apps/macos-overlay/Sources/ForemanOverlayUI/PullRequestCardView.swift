import SwiftUI

import ForemanOverlayCore

struct PullRequestCardView: View {
    let pullRequest: ControlPullRequest
    let accent: Color
    let onOpen: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                Text("Pull Request")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                Text("#\(pullRequest.number)")
                    .font(.caption.monospaced().weight(.bold))
                Text(pullRequest.statusLabel)
                    .font(.caption2.monospaced().weight(.semibold))
                    .foregroundStyle(prStatusColor(pullRequest.status))
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(prStatusColor(pullRequest.status).opacity(0.12), in: Capsule())
                Spacer()
                Button("Open PR") { onOpen() }
                    .buttonStyle(.link)
                    .font(.caption.weight(.semibold))
            }
            Text(pullRequest.title)
                .font(.callout.weight(.medium))
                .lineLimit(2)
            Text("\(pullRequest.repository) · \(pullRequest.branch) → \(pullRequest.baseBranch) · @\(pullRequest.author)")
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(1)
        }
        .padding(10)
        .background(accent.opacity(0.10), in: RoundedRectangle(cornerRadius: 10))
        .overlay(
            RoundedRectangle(cornerRadius: 10)
                .stroke(accent.opacity(0.20), lineWidth: 1)
        )
    }
}
