import Foundation

public enum OverlayTheme: String, CaseIterable, Sendable {
    case system
    case graphite
    case indigo
    case terminal

    public var label: String {
        switch self {
        case .system: "System"
        case .graphite: "Graphite"
        case .indigo: "Indigo"
        case .terminal: "Terminal"
        }
    }
}

public enum OverlayFocusRegion: String, CaseIterable, Sendable {
    case list
    case pullRequest
    case detail
    case compose

    public var label: String {
        switch self {
        case .list: "List"
        case .pullRequest: "PR"
        case .detail: "Details"
        case .compose: "Compose"
        }
    }
}

public enum OverlaySortMode: String, CaseIterable, Identifiable, Sendable {
    case stable
    case attentionFirst
    case recentFirst

    public var id: String { rawValue }
    public var label: String {
        switch self {
        case .stable: "Stable"
        case .attentionFirst: "Attention → Recent"
        case .recentFirst: "Recent First"
        }
    }
}

public enum OverlayHarnessFilter: String, CaseIterable, Identifiable, Sendable {
    case all
    case pi
    case claude
    case codex
    case gemini
    case opencode
    case unknown

    public var id: String { rawValue }
    public var label: String {
        switch self {
        case .all: "All"
        case .pi: "Pi"
        case .claude: "Claude Code"
        case .codex: "Codex CLI"
        case .gemini: "Gemini"
        case .opencode: "OpenCode"
        case .unknown: "Unknown"
        }
    }

    func matches(_ entry: AgentEntry) -> Bool {
        switch self {
        case .all: true
        case .unknown: entry.harnessLabel == nil || entry.harnessLabel == "Unknown"
        default: (entry.harnessLabel ?? "").lowercased().contains(label.lowercased().split(separator: " ").first.map(String.init) ?? rawValue)
        }
    }
}

public enum OverlaySizePreset: String, CaseIterable, Identifiable, Sendable {
    case compact
    case `default`
    case large
    case extraLarge

    public var id: String { rawValue }
    public var label: String {
        switch self {
        case .compact: "Compact"
        case .default: "Default"
        case .large: "Large"
        case .extraLarge: "Extra Large"
        }
    }
    public var width: Double {
        switch self {
        case .compact: 760
        case .default: 900
        case .large: 1100
        case .extraLarge: 1280
        }
    }
    public var height: Double {
        switch self {
        case .compact: 500
        case .default: 600
        case .large: 720
        case .extraLarge: 840
        }
    }
    public var dimensionsLabel: String { "\(Int(width)) × \(Int(height))" }
}

public enum OverlayRowDisplayMode: String, CaseIterable, Identifiable, Sendable {
    case smart
    case workspace
    case session
    case window
    case paneTitle

    public var id: String { rawValue }
    public var label: String {
        switch self {
        case .smart: "Smart"
        case .workspace: "Workspace"
        case .session: "Session"
        case .window: "Window"
        case .paneTitle: "Pane Title"
        }
    }
}
