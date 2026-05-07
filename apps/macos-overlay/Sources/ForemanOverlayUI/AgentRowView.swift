import SwiftUI

import ForemanOverlayCore

struct AgentRow: View {
    let entry: AgentEntry
    let presentation: OverlayRowPresentation
    let flashLabel: String?
    let flashQuery: String

    var body: some View {
        HStack(spacing: 10) {
            if let flashLabel {
                Text(flashLabel)
                    .font(.caption2.monospaced().weight(.bold))
                    .foregroundStyle(flashMatches(label: flashLabel) ? .white : .primary)
                    .frame(width: 24)
                    .padding(.vertical, 3)
                    .background(flashMatches(label: flashLabel) ? Color.blue : Color.secondary.opacity(0.22), in: RoundedRectangle(cornerRadius: 5))
            }
            Circle()
                .fill(statusColor(entry.status))
                .frame(width: 9, height: 9)
            VStack(alignment: .leading, spacing: 3) {
                Text(presentation.title)
                    .font(.system(size: 13, weight: .semibold))
                    .lineLimit(1)
                Text(presentation.subtitle)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer()
            Text(entry.statusLabel)
                .font(.caption2.monospaced().weight(.semibold))
                .foregroundStyle(statusColor(entry.status))
        }
        .padding(.vertical, 4)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("\(presentation.title), \(entry.statusLabel), \(presentation.subtitle)")
        .accessibilityValue(entry.pullRequest == nil ? "No pull request" : "Has pull request")
    }

    private func flashMatches(label: String) -> Bool {
        !flashQuery.isEmpty && label.lowercased().hasPrefix(flashQuery.lowercased())
    }
}

struct StatusBadge: View {
    let status: String
    let label: String

    var body: some View {
        Text(label)
            .font(.caption.monospaced().weight(.bold))
            .foregroundStyle(statusColor(status))
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(statusColor(status).opacity(0.12), in: Capsule())
            .accessibilityLabel("Status: \(label)")
    }
}

struct FlashPromptChip: View {
    let prompt: String
    let focuses: Bool

    var body: some View {
        Text("Flash: \(prompt)" + (focuses ? " → focus" : ""))
            .font(.caption2.monospaced().weight(.semibold))
            .foregroundStyle(.white)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(.blue.opacity(0.90), in: Capsule())
            .accessibilityLabel("Flash jump: \(prompt)")
    }
}

struct ActiveRegionChip: View {
    let region: OverlayFocusRegion

    var body: some View {
        Text("Region: \(region.label)")
            .font(.caption2.weight(.semibold))
            .foregroundStyle(.secondary)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(.quaternary, in: Capsule())
            .accessibilityLabel("Active region: \(region.label)")
    }
}
