import Foundation

public struct AgentsResponse: Decodable, Sendable {
    public let schemaVersion: Int
    public let inventory: InventorySummary
    public let entries: [AgentEntry]
    public let diagnostics: [ControlDiagnostic]
    public let sourceDiagnostics: [ControlDiagnostic]

    public init(schemaVersion: Int, inventory: InventorySummary, entries: [AgentEntry], diagnostics: [ControlDiagnostic], sourceDiagnostics: [ControlDiagnostic] = []) {
        self.schemaVersion = schemaVersion
        self.inventory = inventory
        self.entries = entries
        self.diagnostics = diagnostics
        self.sourceDiagnostics = sourceDiagnostics
    }

    public var allDiagnostics: [ControlDiagnostic] {
        diagnostics + sourceDiagnostics
    }

    public func mergingExtensionCards(from extensionResponse: AgentsResponse) -> AgentsResponse {
        let cardsBySourcePane = Dictionary(uniqueKeysWithValues: extensionResponse.entries.map { ($0.sourcePaneId, $0.extensionCards) })
        let mergedEntries = entries.map { entry in
            guard let cards = cardsBySourcePane[entry.sourcePaneId] else { return entry }
            return entry.withExtensionCards(cards)
        }
        return AgentsResponse(schemaVersion: schemaVersion, inventory: inventory, entries: mergedEntries, diagnostics: diagnostics, sourceDiagnostics: sourceDiagnostics)
    }

    public func mergingExtensionCards(_ cards: [ControlExtensionCard], forSourcePaneId sourcePaneId: String) -> AgentsResponse {
        let mergedEntries = entries.map { entry in
            entry.sourcePaneId == sourcePaneId ? entry.withExtensionCards(cards) : entry
        }
        return AgentsResponse(schemaVersion: schemaVersion, inventory: inventory, entries: mergedEntries, diagnostics: diagnostics, sourceDiagnostics: sourceDiagnostics)
    }

    private enum CodingKeys: String, CodingKey {
        case schemaVersion, inventory, entries, diagnostics, sourceDiagnostics
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        schemaVersion = try container.decode(Int.self, forKey: .schemaVersion)
        inventory = try container.decode(InventorySummary.self, forKey: .inventory)
        entries = try container.decode([AgentEntry].self, forKey: .entries)
        diagnostics = try container.decodeIfPresent([ControlDiagnostic].self, forKey: .diagnostics) ?? []
        sourceDiagnostics = try container.decodeIfPresent([ControlDiagnostic].self, forKey: .sourceDiagnostics) ?? []
    }
}

public struct ExtensionCardsResponse: Decodable, Sendable {
    public let schemaVersion: Int
    public let ok: Bool
    public let action: String
    public let paneId: String
    public let sourceId: String
    public let sourcePaneId: String
    public let workspace: String?
    public let linkedRepository: String?
    public let targetPath: String
    public let extensionCards: [ControlExtensionCard]

    private enum CodingKeys: String, CodingKey {
        case schemaVersion, ok, action, paneId, sourceId, sourcePaneId, workspace, linkedRepository, targetPath, extensionCards
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        schemaVersion = try container.decode(Int.self, forKey: .schemaVersion)
        ok = try container.decode(Bool.self, forKey: .ok)
        action = try container.decode(String.self, forKey: .action)
        paneId = try container.decode(String.self, forKey: .paneId)
        sourceId = try container.decodeIfPresent(String.self, forKey: .sourceId) ?? AgentEntry.defaultSourceId
        sourcePaneId = try container.decodeIfPresent(String.self, forKey: .sourcePaneId) ?? AgentEntry.defaultSourcePaneId(sourceId: sourceId, paneId: paneId)
        workspace = try container.decodeIfPresent(String.self, forKey: .workspace)
        linkedRepository = try container.decodeIfPresent(String.self, forKey: .linkedRepository)
        targetPath = try container.decode(String.self, forKey: .targetPath)
        extensionCards = try container.decodeIfPresent([ControlExtensionCard].self, forKey: .extensionCards) ?? []
    }
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
    public var id: String { [sourceId, code, level, message].compactMap { $0 }.joined(separator: ":") }
    public let level: String
    public let code: String?
    public let sourceId: String?
    public let sourceLabel: String?
    public let sourceKind: String?
    public let message: String
    public let retryable: Bool?
    public let durationMs: UInt64?
    public let lastSuccessUnixMs: UInt64?

    public init(level: String, code: String? = nil, sourceId: String? = nil, sourceLabel: String? = nil, sourceKind: String? = nil, message: String, retryable: Bool? = nil, durationMs: UInt64? = nil, lastSuccessUnixMs: UInt64? = nil) {
        self.level = level
        self.code = code
        self.sourceId = sourceId
        self.sourceLabel = sourceLabel
        self.sourceKind = sourceKind
        self.message = message
        self.retryable = retryable
        self.durationMs = durationMs
        self.lastSuccessUnixMs = lastSuccessUnixMs
    }
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

public struct ControlExtensionCard: Decodable, Identifiable, Sendable {
    public let id: String
    public let title: String
    public let status: String
    public let statusLabel: String
    public let summary: String
    public let rows: [ControlExtensionRow]
    public let actions: [ControlExtensionAction]
}

public struct ControlExtensionRow: Decodable, Sendable {
    public let label: String
    public let value: String
    public let status: String?
}

public struct ControlExtensionAction: Decodable, Identifiable, Sendable {
    public let id: String
    public let label: String
    public let kind: String
    public let value: String
}

public struct AgentEntry: Decodable, Identifiable, Sendable {
    public static let defaultSourceId = "local"
    public static let defaultSourceLabel = "Local"
    public static let defaultSourceKind = "local"

    public var id: String { sourcePaneId }
    public let controlId: String?
    public let sourcePaneId: String
    public let sourceId: String
    public let sourceLabel: String
    public let sourceDisplayLabel: String
    public let sourceShowLabel: Bool
    public let sourceKind: String
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
    public let linkedRepository: String?
    public let workspaceName: String?
    public let preview: String
    public let previewProvenance: String
    public let activityScore: UInt64
    public let statusRank: Int?
    public let lastActivityUnixMs: UInt64?
    public let lastStatusChangeUnixMs: UInt64?
    public let activeRunCount: UInt32?
    public let pullRequest: ControlPullRequest?
    public let extensionCards: [ControlExtensionCard]

    public init(
        id: String,
        sourcePaneId: String? = nil,
        sourceId: String = AgentEntry.defaultSourceId,
        sourceLabel: String = AgentEntry.defaultSourceLabel,
        sourceDisplayLabel: String? = nil,
        sourceShowLabel: Bool = false,
        sourceKind: String = AgentEntry.defaultSourceKind,
        paneId: String,
        sessionName: String,
        windowName: String,
        title: String,
        navigationTitle: String,
        harness: String?,
        harnessLabel: String?,
        status: String,
        statusLabel: String,
        statusSource: String?,
        integrationMode: String?,
        isAgent: Bool,
        currentCommand: String?,
        runtimeCommand: String?,
        workingDir: String?,
        linkedRepository: String?,
        workspaceName: String?,
        preview: String,
        previewProvenance: String,
        activityScore: UInt64,
        statusRank: Int?,
        lastActivityUnixMs: UInt64?,
        lastStatusChangeUnixMs: UInt64?,
        activeRunCount: UInt32?,
        pullRequest: ControlPullRequest?,
        extensionCards: [ControlExtensionCard]
    ) {
        self.controlId = id
        self.paneId = paneId
        self.sourceId = sourceId
        self.sourceLabel = sourceLabel
        self.sourceDisplayLabel = sourceDisplayLabel ?? sourceLabel
        self.sourceShowLabel = sourceShowLabel
        self.sourceKind = sourceKind
        self.sourcePaneId = sourcePaneId ?? Self.defaultSourcePaneId(sourceId: sourceId, paneId: paneId)
        self.sessionName = sessionName
        self.windowName = windowName
        self.title = title
        self.navigationTitle = navigationTitle
        self.harness = harness
        self.harnessLabel = harnessLabel
        self.status = status
        self.statusLabel = statusLabel
        self.statusSource = statusSource
        self.integrationMode = integrationMode
        self.isAgent = isAgent
        self.currentCommand = currentCommand
        self.runtimeCommand = runtimeCommand
        self.workingDir = workingDir
        self.linkedRepository = linkedRepository
        self.workspaceName = workspaceName
        self.preview = preview
        self.previewProvenance = previewProvenance
        self.activityScore = activityScore
        self.statusRank = statusRank
        self.lastActivityUnixMs = lastActivityUnixMs
        self.lastStatusChangeUnixMs = lastStatusChangeUnixMs
        self.activeRunCount = activeRunCount
        self.pullRequest = pullRequest
        self.extensionCards = extensionCards
    }

    public static func defaultSourcePaneId(sourceId: String, paneId: String) -> String {
        "source:\(sourceId):pane:\(paneId)"
    }

    public var isDefaultSource: Bool {
        sourceId == Self.defaultSourceId && sourceKind == Self.defaultSourceKind
    }

    public func withExtensionCards(_ extensionCards: [ControlExtensionCard]) -> AgentEntry {
        AgentEntry(
            id: controlId ?? sourcePaneId,
            sourcePaneId: sourcePaneId,
            sourceId: sourceId,
            sourceLabel: sourceLabel,
            sourceDisplayLabel: sourceDisplayLabel,
            sourceShowLabel: sourceShowLabel,
            sourceKind: sourceKind,
            paneId: paneId,
            sessionName: sessionName,
            windowName: windowName,
            title: title,
            navigationTitle: navigationTitle,
            harness: harness,
            harnessLabel: harnessLabel,
            status: status,
            statusLabel: statusLabel,
            statusSource: statusSource,
            integrationMode: integrationMode,
            isAgent: isAgent,
            currentCommand: currentCommand,
            runtimeCommand: runtimeCommand,
            workingDir: workingDir,
            linkedRepository: linkedRepository,
            workspaceName: workspaceName,
            preview: preview,
            previewProvenance: previewProvenance,
            activityScore: activityScore,
            statusRank: statusRank,
            lastActivityUnixMs: lastActivityUnixMs,
            lastStatusChangeUnixMs: lastStatusChangeUnixMs,
            activeRunCount: activeRunCount,
            pullRequest: pullRequest,
            extensionCards: extensionCards
        )
    }

    private enum CodingKeys: String, CodingKey {
        case id, sourcePaneId, sourceId, sourceLabel, sourceDisplayLabel, sourceShowLabel, sourceKind
        case paneId, sessionName, windowName, title, navigationTitle, harness, harnessLabel
        case status, statusLabel, statusSource, integrationMode, isAgent, currentCommand
        case runtimeCommand, workingDir, linkedRepository, workspaceName, preview, previewProvenance, activityScore
        case statusRank, lastActivityUnixMs, lastStatusChangeUnixMs, activeRunCount
        case pullRequest, extensionCards
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        controlId = try container.decodeIfPresent(String.self, forKey: .id)
        paneId = try container.decode(String.self, forKey: .paneId)
        sourceId = try container.decodeIfPresent(String.self, forKey: .sourceId) ?? Self.defaultSourceId
        sourceLabel = try container.decodeIfPresent(String.self, forKey: .sourceLabel) ?? Self.defaultSourceLabel
        sourceDisplayLabel = try container.decodeIfPresent(String.self, forKey: .sourceDisplayLabel) ?? sourceLabel
        sourceShowLabel = try container.decodeIfPresent(Bool.self, forKey: .sourceShowLabel) ?? false
        sourceKind = try container.decodeIfPresent(String.self, forKey: .sourceKind) ?? Self.defaultSourceKind
        sourcePaneId = try container.decodeIfPresent(String.self, forKey: .sourcePaneId) ?? Self.defaultSourcePaneId(sourceId: sourceId, paneId: paneId)
        sessionName = try container.decode(String.self, forKey: .sessionName)
        windowName = try container.decode(String.self, forKey: .windowName)
        title = try container.decode(String.self, forKey: .title)
        navigationTitle = try container.decode(String.self, forKey: .navigationTitle)
        harness = try container.decodeIfPresent(String.self, forKey: .harness)
        harnessLabel = try container.decodeIfPresent(String.self, forKey: .harnessLabel)
        status = try container.decode(String.self, forKey: .status)
        statusLabel = try container.decode(String.self, forKey: .statusLabel)
        statusSource = try container.decodeIfPresent(String.self, forKey: .statusSource)
        integrationMode = try container.decodeIfPresent(String.self, forKey: .integrationMode)
        isAgent = try container.decode(Bool.self, forKey: .isAgent)
        currentCommand = try container.decodeIfPresent(String.self, forKey: .currentCommand)
        runtimeCommand = try container.decodeIfPresent(String.self, forKey: .runtimeCommand)
        workingDir = try container.decodeIfPresent(String.self, forKey: .workingDir)
        linkedRepository = try container.decodeIfPresent(String.self, forKey: .linkedRepository)
        workspaceName = try container.decodeIfPresent(String.self, forKey: .workspaceName)
        preview = try container.decode(String.self, forKey: .preview)
        previewProvenance = try container.decode(String.self, forKey: .previewProvenance)
        activityScore = try container.decode(UInt64.self, forKey: .activityScore)
        statusRank = try container.decodeIfPresent(Int.self, forKey: .statusRank)
        lastActivityUnixMs = try container.decodeIfPresent(UInt64.self, forKey: .lastActivityUnixMs)
        lastStatusChangeUnixMs = try container.decodeIfPresent(UInt64.self, forKey: .lastStatusChangeUnixMs)
        activeRunCount = try container.decodeIfPresent(UInt32.self, forKey: .activeRunCount)
        pullRequest = try container.decodeIfPresent(ControlPullRequest.self, forKey: .pullRequest)
        extensionCards = try container.decodeIfPresent([ControlExtensionCard].self, forKey: .extensionCards) ?? []
    }
}
