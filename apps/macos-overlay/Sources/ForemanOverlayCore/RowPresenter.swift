import Foundation

public struct OverlayRowPresentation: Equatable, Sendable {
    public let title: String
    public let subtitle: String
}

public struct PresentedAgentRow: Identifiable, Sendable {
    public var id: String { entry.id }
    public let entry: AgentEntry
    public let presentation: OverlayRowPresentation
}

public enum OverlayRowPresenter {
    public static func presentedRows(for entries: [AgentEntry], mode: OverlayRowDisplayMode) -> [PresentedAgentRow] {
        let duplicateCounts = workspaceDuplicateCounts(for: entries)
        return entries.map { entry in
            PresentedAgentRow(entry: entry, presentation: presentation(for: entry, mode: mode, duplicateCounts: duplicateCounts))
        }
    }

    public static func presentation(for entry: AgentEntry, in entries: [AgentEntry], mode: OverlayRowDisplayMode) -> OverlayRowPresentation {
        presentation(for: entry, mode: mode, duplicateCounts: workspaceDuplicateCounts(for: entries))
    }

    private static func presentation(for entry: AgentEntry, mode: OverlayRowDisplayMode, duplicateCounts: [String: Int]) -> OverlayRowPresentation {
        let title: String
        switch mode {
        case .smart:
            title = smartTitle(for: entry, duplicateCounts: duplicateCounts)
        case .workspace:
            title = workspaceTitle(entry)
        case .session:
            title = fallback(entry.sessionName, entry.navigationTitle)
        case .window:
            title = fallback(entry.windowName, entry.navigationTitle)
        case .paneTitle:
            title = fallback(cleanPaneTitle(entry.title, entry: entry), entry.navigationTitle)
        }
        return OverlayRowPresentation(title: title, subtitle: subtitle(for: entry, title: title, duplicateCounts: duplicateCounts))
    }

    private static func smartTitle(for entry: AgentEntry, duplicateCounts: [String: Int]) -> String {
        let workspace = workspaceTitle(entry)
        let duplicateCount = duplicateCounts[workspace, default: 0]
        guard duplicateCount > 1 else { return workspace }
        if let window = usefulWindowName(entry) { return window }
        if let paneTitle = cleanPaneTitle(entry.title, entry: entry), paneTitle != workspace { return paneTitle }
        return workspace
    }

    private static func subtitle(for entry: AgentEntry, title: String, duplicateCounts: [String: Int]) -> String {
        var parts = [entry.harnessLabel ?? "Unknown", entry.sessionName]
        let workspace = workspaceTitle(entry)
        let duplicateCount = duplicateCounts[workspace, default: 0]
        if duplicateCount > 1 || title != workspace {
            parts.append(workspace)
            parts.append(entry.paneId)
        } else if title != entry.windowName, let window = usefulWindowName(entry) {
            parts.append(window)
        }
        return parts.filter { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }.joined(separator: " · ")
    }

    private static func workspaceDuplicateCounts(for entries: [AgentEntry]) -> [String: Int] {
        Dictionary(grouping: entries, by: workspaceTitle).mapValues(\.count)
    }

    private static func workspaceTitle(_ entry: AgentEntry) -> String {
        fallback(entry.workspaceName, entry.navigationTitle)
    }

    private static func usefulWindowName(_ entry: AgentEntry) -> String? {
        let trimmed = entry.windowName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }
        let generic = ["zsh", "bash", "fish", "sh", "node", "python", "nvim", "vim"]
        return generic.contains(trimmed.lowercased()) ? nil : trimmed
    }

    private static func cleanPaneTitle(_ title: String, entry: AgentEntry) -> String? {
        let trimmed = title.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, trimmed != entry.paneId else { return nil }
        return trimmed
            .replacingOccurrences(of: "π - ", with: "")
            .replacingOccurrences(of: "⠋ ", with: "")
            .replacingOccurrences(of: "⠇ ", with: "")
            .replacingOccurrences(of: "⠸ ", with: "")
            .trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private static func fallback(_ candidate: String?, _ fallback: String) -> String {
        let trimmed = candidate?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return trimmed.isEmpty ? fallback : trimmed
    }
}
