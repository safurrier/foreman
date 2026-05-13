import AppKit
import SwiftUI

import ForemanOverlayCore

struct ExtensionCardView: View {
    let card: ControlExtensionCard
    let accent: Color
    let performAction: (ControlExtensionAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                Text(card.title)
                    .font(.caption.weight(.semibold))
                Spacer()
                ExtensionStatusBadge(status: card.status, label: card.statusLabel)
            }

            Text(card.summary)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(2)

            ForEach(Array(card.rows.prefix(6).enumerated()), id: \.offset) { _, row in
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text(row.label)
                        .font(.caption2.weight(.medium))
                        .foregroundStyle(.secondary)
                        .frame(width: 78, alignment: .trailing)
                    Text(row.value)
                        .font(.caption2.monospaced())
                        .foregroundStyle(color(for: row.status))
                        .lineLimit(2)
                    Spacer(minLength: 0)
                }
            }

            if !card.actions.isEmpty {
                HStack(spacing: 8) {
                    ForEach(card.actions.prefix(3)) { action in
                        Button(action.label) { performAction(action) }
                            .font(.caption2)
                            .buttonStyle(.borderless)
                            .foregroundStyle(accent)
                    }
                    Spacer(minLength: 0)
                }
                .padding(.top, 2)
            }
        }
        .padding(10)
        .background(.quaternary.opacity(0.28), in: RoundedRectangle(cornerRadius: 12))
    }

    private func color(for status: String?) -> Color {
        switch status {
        case "fail", "not-ready", "unavailable", "error": .red
        case "working", "refreshing", "attention", "needs-validation", "needs-review", "needs-sync", "needs-attention": .orange
        case "pass", "ready", "synced": .green
        case "info", "idle": .secondary
        default: .primary
        }
    }
}

private struct ExtensionStatusBadge: View {
    let status: String
    let label: String

    var body: some View {
        Text(label)
            .font(.caption2.weight(.bold))
            .padding(.horizontal, 7)
            .padding(.vertical, 3)
            .foregroundStyle(.white)
            .background(color, in: Capsule())
    }

    private var color: Color {
        switch status {
        case "ready", "pass", "synced": .green
        case "fail", "not-ready", "unavailable", "error": .red
        case "working", "refreshing", "attention", "needs-attention", "needs-validation", "needs-review", "needs-sync": .orange
        case "idle", "info": .gray
        default: .blue
        }
    }
}
