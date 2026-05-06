import Combine
import Foundation

public enum OverlayStatusFilter: String, CaseIterable, Identifiable, Sendable {
    case all
    case working
    case needsAttention
    case idle
    case error

    public var id: String { rawValue }
    public var label: String {
        switch self {
        case .all: "All"
        case .working: "Working"
        case .needsAttention: "Needs Attention"
        case .idle: "Idle"
        case .error: "Error"
        }
    }

    func matches(_ entry: AgentEntry) -> Bool {
        switch self {
        case .all: true
        case .needsAttention: entry.status == "needs-attention"
        default: entry.status == rawValue
        }
    }
}

@MainActor
public final class OverlayPreferences: ObservableObject {
    public static let defaultPopupWidth = OverlaySizePreset.default.width
    public static let defaultPopupHeight = OverlaySizePreset.default.height
    public static let minPopupWidth = 720.0
    public static let maxPopupWidth = 1400.0
    public static let minPopupHeight = 480.0
    public static let maxPopupHeight = 1000.0

    private let defaults: UserDefaults
    private let prefix = "ForemanOverlay."

    @Published public var sizePreset: OverlaySizePreset { didSet { save("sizePreset", sizePreset.rawValue); popupWidth = sizePreset.width; popupHeight = sizePreset.height } }
    @Published public var popupWidth: Double { didSet { save("popupWidth", Self.clamp(popupWidth, Self.minPopupWidth, Self.maxPopupWidth)) } }
    @Published public var popupHeight: Double { didSet { save("popupHeight", Self.clamp(popupHeight, Self.minPopupHeight, Self.maxPopupHeight)) } }
    @Published public var includeAllPanes: Bool { didSet { save("includeAllPanes", includeAllPanes) } }
    @Published public var sortMode: OverlaySortMode { didSet { save("sortMode", sortMode.rawValue) } }
    @Published public var harnessFilter: OverlayHarnessFilter { didSet { save("harnessFilter", harnessFilter.rawValue) } }
    @Published public var statusFilter: OverlayStatusFilter { didSet { save("statusFilter", statusFilter.rawValue) } }
    @Published public var rowDisplayMode: OverlayRowDisplayMode { didSet { save("rowDisplayMode", rowDisplayMode.rawValue) } }
    @Published public var terminalActivation: String { didSet { save("terminalActivation", terminalActivation) } }
    @Published public var customTerminalBundleID: String { didSet { save("customTerminalBundleID", customTerminalBundleID) } }

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        let loadedSizePreset = OverlaySizePreset(rawValue: defaults.string(forKey: prefix + "sizePreset") ?? "") ?? .default
        sizePreset = loadedSizePreset
        popupWidth = Self.clamp(defaults.object(forKey: prefix + "popupWidth") as? Double ?? loadedSizePreset.width, Self.minPopupWidth, Self.maxPopupWidth)
        popupHeight = Self.clamp(defaults.object(forKey: prefix + "popupHeight") as? Double ?? loadedSizePreset.height, Self.minPopupHeight, Self.maxPopupHeight)
        includeAllPanes = defaults.bool(forKey: prefix + "includeAllPanes")
        sortMode = OverlaySortMode(rawValue: defaults.string(forKey: prefix + "sortMode") ?? "") ?? .stable
        harnessFilter = OverlayHarnessFilter(rawValue: defaults.string(forKey: prefix + "harnessFilter") ?? "") ?? .all
        statusFilter = OverlayStatusFilter(rawValue: defaults.string(forKey: prefix + "statusFilter") ?? "") ?? .all
        rowDisplayMode = OverlayRowDisplayMode(rawValue: defaults.string(forKey: prefix + "rowDisplayMode") ?? "") ?? .smart
        terminalActivation = defaults.string(forKey: prefix + "terminalActivation") ?? "auto"
        customTerminalBundleID = defaults.string(forKey: prefix + "customTerminalBundleID") ?? ""
    }

    public func resetPopupSize() {
        sizePreset = .default
        popupWidth = Self.defaultPopupWidth
        popupHeight = Self.defaultPopupHeight
    }

    public var activeFilterSummary: String? {
        var parts: [String] = []
        if includeAllPanes { parts.append("All panes") }
        if sortMode != .stable { parts.append(sortMode.label) }
        if harnessFilter != .all { parts.append(harnessFilter.label) }
        if statusFilter != .all { parts.append(statusFilter.label) }
        if rowDisplayMode != .smart { parts.append("Rows: \(rowDisplayMode.label)") }
        return parts.isEmpty ? nil : parts.joined(separator: " · ")
    }

    private func save(_ key: String, _ value: Any) {
        defaults.set(value, forKey: prefix + key)
    }

    private static func clamp(_ value: Double, _ minValue: Double, _ maxValue: Double) -> Double {
        min(max(value, minValue), maxValue)
    }
}
