import Foundation

public struct AgentsResponse: Decodable, Sendable {
    public let schemaVersion: Int
    public let inventory: InventorySummary
    public let entries: [AgentEntry]
    public let diagnostics: [ControlDiagnostic]
}

public struct InventorySummary: Decodable, Sendable {
    public let totalSessions: Int
    public let totalWindows: Int
    public let totalPanes: Int
    public let visibleSessions: Int
    public let visibleWindows: Int
    public let visiblePanes: Int
}

public struct ControlDiagnostic: Decodable, Identifiable, Sendable {
    public var id: String { "\(level):\(message)" }
    public let level: String
    public let message: String
}

public struct ControlPullRequest: Decodable, Sendable {
    public let number: UInt64
    public let title: String
    public let url: String
    public let repository: String
    public let branch: String
    public let baseBranch: String
    public let author: String
    public let status: String
    public let statusLabel: String
}

public struct AgentEntry: Decodable, Identifiable, Sendable {
    public let id: String
    public let paneId: String
    public let sessionName: String
    public let windowName: String
    public let title: String
    public let navigationTitle: String
    public let harness: String?
    public let harnessLabel: String?
    public let status: String
    public let statusLabel: String
    public let statusSource: String?
    public let integrationMode: String?
    public let isAgent: Bool
    public let currentCommand: String?
    public let runtimeCommand: String?
    public let workingDir: String?
    public let workspaceName: String?
    public let preview: String
    public let previewProvenance: String
    public let activityScore: UInt64
    public let pullRequest: ControlPullRequest?
}
