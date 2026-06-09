import Combine
import Foundation

@MainActor
public final class OverlayStore: ObservableObject {
    @Published public var query = "" {
        didSet { normalizeSelection() }
    }
    @Published public var response: AgentsResponse? {
        didSet { normalizeSelection() }
    }
    @Published public var selectionId: String? {
        didSet {
            if activeRegion == .pullRequest, selectedEntry?.pullRequest == nil {
                activeRegion = .detail
            }
            scheduleSelectedExtensionLookup()
        }
    }
    @Published public var isLoading = false
    @Published public var isLoadingExtensions = false
    @Published public var extensionErrorMessage: String?
    @Published public var extensionErrorPaneId: String?
    @Published public var extensionLoadingPaneId: String?
    @Published public var errorMessage: String?
    @Published public var composeText = ""
    @Published public var isComposing = false
    @Published public var activeRegion: OverlayFocusRegion = .list
    @Published public var isHelpVisible = false
    @Published public var helpScrollOffset = 0
    @Published public var previewScrollOffset = 0
    @Published public var theme: OverlayTheme = .system
    @Published public var lastReloadSummary: String?
    @Published public var isFlashVisible = false
    @Published public var flashQuery = ""
    @Published public var flashFocusesOnMatch = false

    public weak var appRouter: OverlayAppRouting?

    public let preferences: OverlayPreferences
    private var client: ForemanClient
    private var preferenceCancellable: AnyCancellable?
    private var reloadTask: Task<Void, Never>?
    private var extensionLookupTask: Task<Void, Never>?
    private var preferenceReloadTask: Task<Void, Never>?
    private var reloadGeneration = 0
    private var extensionLookupGeneration = 0
    private var extensionLoadedPaneIds = Set<String>()
    private var lastIncludeAllPanes: Bool

    public init(client: ForemanClient, preferences: OverlayPreferences = OverlayPreferences()) {
        self.client = client
        self.preferences = preferences
        self.lastIncludeAllPanes = preferences.includeAllPanes
        preferenceCancellable = preferences.objectWillChange.sink { [weak self] _ in
            Task { @MainActor in
                try? await Task.sleep(for: .milliseconds(25))
                guard let self else { return }
                self.objectWillChange.send()
                self.normalizeSelection()
                if self.preferences.includeAllPanes != self.lastIncludeAllPanes {
                    self.lastIncludeAllPanes = self.preferences.includeAllPanes
                    self.schedulePreferenceReload()
                }
            }
        }
    }

    public var entries: [AgentEntry] {
        guard let response else { return [] }
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        var visible = response.entries.filter { entry in
            preferences.harnessFilter.matches(entry) && preferences.statusFilter.matches(entry)
        }
        if !trimmed.isEmpty {
            visible = visible.filter { entry in
                [
                    entry.navigationTitle,
                    entry.sessionName,
                    entry.windowName,
                    entry.harnessLabel ?? "",
                    entry.status,
                    entry.sourceLabel,
                    entry.sourceId,
                    entry.workingDir ?? "",
                    entry.preview,
                ].joined(separator: " ").lowercased().contains(trimmed)
            }
        }
        switch preferences.sortMode {
        case .stable:
            return visible
        case .attentionFirst:
            return visible.sorted { lhs, rhs in
                let lhsPriority = lhs.statusRank ?? statusPriority(lhs.status)
                let rhsPriority = rhs.statusRank ?? statusPriority(rhs.status)
                if lhsPriority != rhsPriority { return lhsPriority < rhsPriority }
                let lhsActivity = lhs.lastActivityUnixMs ?? lhs.activityScore
                let rhsActivity = rhs.lastActivityUnixMs ?? rhs.activityScore
                if lhsActivity != rhsActivity { return lhsActivity > rhsActivity }
                if lhs.navigationTitle != rhs.navigationTitle { return lhs.navigationTitle < rhs.navigationTitle }
                return lhs.id < rhs.id
            }
        case .recentFirst:
            return visible.sorted { lhs, rhs in
                let lhsActivity = lhs.lastActivityUnixMs ?? lhs.activityScore
                let rhsActivity = rhs.lastActivityUnixMs ?? rhs.activityScore
                if lhsActivity != rhsActivity { return lhsActivity > rhsActivity }
                if lhs.navigationTitle != rhs.navigationTitle { return lhs.navigationTitle < rhs.navigationTitle }
                return lhs.id < rhs.id
            }
        }
    }

    public var presentedEntries: [PresentedAgentRow] {
        OverlayRowPresenter.presentedRows(for: entries, mode: preferences.rowDisplayMode)
    }

    public var selectedEntry: AgentEntry? {
        guard let selectionId else { return entries.first }
        return entries.first { $0.id == selectionId }
    }

    public var selectedEntryIsLoadingExtensions: Bool {
        guard let selectedEntry else { return false }
        return isLoadingExtensions && extensionLoadingPaneId == selectedEntry.sourcePaneId
    }

    public var selectedEntryExtensionError: String? {
        guard let selectedEntry, extensionErrorPaneId == selectedEntry.sourcePaneId else { return nil }
        return extensionErrorMessage
    }

    private func normalizeSelection() {
        let visible = entries
        let normalized = visible.contains { $0.id == selectionId } ? selectionId : visible.first?.id
        if selectionId != normalized {
            selectionId = normalized
        }
    }

    private func statusPriority(_ status: String) -> Int {
        switch status {
        case "error": 0
        case "needs-attention": 1
        case "working": 2
        case "idle": 3
        default: 4
        }
    }

    public var footerHint: String {
        if isHelpVisible {
            return "Help • ↑/↓ or j/k scroll • Esc close"
        }
        if isFlashVisible {
            return flashFocusesOnMatch
                ? "Flash focus • type label to focus • Esc cancel"
                : "Flash jump • type label to select • Enter focus • Esc cancel"
        }
        switch activeRegion {
        case .list:
            return "List • type search • ↑/↓ move • Enter focus • Cmd+J jump • ? help • Cmd+, settings • Cmd+T theme"
        case .pullRequest:
            return "PR • Enter open in browser • Tab details • Shift+Tab list • Cmd+J jump • Cmd+, settings"
        case .detail:
            return "Details • ↑/↓ scroll preview • Enter focus • Cmd+J jump • Tab compose • Cmd+, settings • Cmd+T theme"
        case .compose:
            return "Compose • Cmd+Enter send • Esc cancel • Shift+Tab details"
        }
    }

    private func schedulePreferenceReload() {
        preferenceReloadTask?.cancel()
        preferenceReloadTask = Task { [weak self] in
            try? await Task.sleep(for: .milliseconds(150))
            guard let self, !Task.isCancelled else { return }
            self.reload()
        }
    }

    public func reload() {
        reloadGeneration += 1
        extensionLookupGeneration += 1
        let generation = reloadGeneration
        reloadTask?.cancel()
        extensionLookupTask?.cancel()
        extensionLoadedPaneIds.removeAll()
        isLoading = true
        isLoadingExtensions = false
        extensionLoadingPaneId = nil
        errorMessage = nil
        extensionErrorMessage = nil
        extensionErrorPaneId = nil
        client.includeAllPanes = preferences.includeAllPanes
        let loadClient = client
        reloadTask = Task { [weak self] in
            do {
                let loaded = try await loadClient.initialAgents()
                guard let self, !Task.isCancelled, generation == self.reloadGeneration else { return }
                response = loaded
                lastReloadSummary = "Loaded \(loaded.entries.count) agent session(s)"
                NSLog("Foreman overlay loaded \(loaded.entries.count) visible entries; total panes: \(loaded.inventory.totalPanes)")
                if selectionId == nil || !loaded.entries.contains(where: { $0.id == selectionId }) {
                    selectionId = loaded.entries.first?.id
                }
                isLoading = false
                scheduleSelectedExtensionLookup()
            } catch is CancellationError {
                guard let self, generation == self.reloadGeneration else { return }
                isLoading = false
            } catch {
                guard let self, !Task.isCancelled, generation == self.reloadGeneration else { return }
                errorMessage = error.localizedDescription
                lastReloadSummary = "Reload failed: \(error.localizedDescription)"
                NSLog("Foreman overlay reload failed: \(error.localizedDescription)")
                isLoading = false
            }
        }
    }

    private func scheduleSelectedExtensionLookup() {
        guard response != nil else { return }
        guard let entry = selectedEntry else {
            extensionLookupGeneration += 1
            extensionLookupTask?.cancel()
            isLoadingExtensions = false
            extensionLoadingPaneId = nil
            extensionErrorMessage = nil
            extensionErrorPaneId = nil
            return
        }
        extensionErrorMessage = nil
        extensionErrorPaneId = nil
        guard entry.extensionCards.isEmpty else { return }
        guard !extensionLoadedPaneIds.contains(entry.sourcePaneId) else { return }
        extensionLookupGeneration += 1
        let lookupGeneration = extensionLookupGeneration
        let reloadGeneration = reloadGeneration
        let paneId = entry.paneId
        let sourcePaneId = entry.sourcePaneId
        let sourceId = entry.sourceId
        extensionLookupTask?.cancel()
        isLoadingExtensions = true
        extensionLoadingPaneId = sourcePaneId
        extensionErrorMessage = nil
        let loadClient = client
        extensionLookupTask = Task { [weak self] in
            do {
                try await Task.sleep(for: .milliseconds(120))
                let loaded = try await loadClient.extensionCards(forPane: paneId, sourceId: sourceId)
                guard let self, !Task.isCancelled, reloadGeneration == self.reloadGeneration, lookupGeneration == self.extensionLookupGeneration else { return }
                guard self.selectedEntry?.sourcePaneId == sourcePaneId else { return }
                response = response?.mergingExtensionCards(loaded.extensionCards, forSourcePaneId: sourcePaneId)
                extensionLoadedPaneIds.insert(sourcePaneId)
                isLoadingExtensions = false
                extensionLoadingPaneId = nil
            } catch is CancellationError {
                guard let self, reloadGeneration == self.reloadGeneration, lookupGeneration == self.extensionLookupGeneration else { return }
                isLoadingExtensions = false
                extensionLoadingPaneId = nil
            } catch {
                guard let self, !Task.isCancelled, reloadGeneration == self.reloadGeneration, lookupGeneration == self.extensionLookupGeneration else { return }
                extensionErrorMessage = error.localizedDescription
                extensionErrorPaneId = sourcePaneId
                NSLog("Foreman overlay extension lookup failed for \(sourcePaneId): \(error.localizedDescription)")
                isLoadingExtensions = false
                extensionLoadingPaneId = nil
            }
        }
    }

    public func moveSelection(delta: Int) {
        let list = entries
        guard !list.isEmpty else { return }
        let current = selectionId.flatMap { id in list.firstIndex { $0.id == id } } ?? 0
        let next = min(max(current + delta, 0), list.count - 1)
        selectionId = list[next].id
        previewScrollOffset = 0
    }

    public var availableFocusRegions: [OverlayFocusRegion] {
        selectedEntry?.pullRequest == nil
            ? [.list, .detail, .compose]
            : [.list, .pullRequest, .detail, .compose]
    }

    public func cycleRegion(reverse: Bool = false) {
        let all = availableFocusRegions
        let current = all.firstIndex(of: activeRegion) ?? 0
        let next = reverse ? (current - 1 + all.count) % all.count : (current + 1) % all.count
        activeRegion = all[next]
        isComposing = activeRegion == .compose
    }

    public func activateRegion(_ region: OverlayFocusRegion) {
        activeRegion = availableFocusRegions.contains(region) ? region : .detail
        isComposing = activeRegion == .compose
    }

    public func cycleTheme() {
        let all = OverlayTheme.allCases
        let current = all.firstIndex(of: theme) ?? 0
        theme = all[(current + 1) % all.count]
    }

    public func beginFlash(focusOnMatch: Bool = false) {
        guard !entries.isEmpty else { return }
        isHelpVisible = false
        isComposing = false
        activeRegion = .list
        isFlashVisible = true
        flashQuery = ""
        flashFocusesOnMatch = focusOnMatch
    }

    public func cancelFlash() {
        isFlashVisible = false
        flashQuery = ""
        flashFocusesOnMatch = false
    }

    public func flashLabel(for entry: AgentEntry) -> String? {
        guard isFlashVisible else { return nil }
        guard let index = entries.firstIndex(where: { $0.id == entry.id }) else { return nil }
        return Self.flashLabel(for: index, count: entries.count)
    }

    public var flashPrompt: String? {
        guard isFlashVisible else { return nil }
        return flashQuery.isEmpty ? "Type a label" : flashQuery.uppercased()
    }

    private static func flashLabel(for index: Int, count: Int) -> String {
        let alphabet = Array("asdfghjklqwertyuiopzxcvbnm")
        let width = count <= alphabet.count ? 1 : 2
        var value = index
        var scalars: [Character] = []
        repeat {
            scalars.append(alphabet[value % alphabet.count])
            value /= alphabet.count
        } while value > 0
        while scalars.count < width {
            scalars.append(alphabet[0])
        }
        return String(scalars.reversed()).uppercased()
    }

    public func toggleHelp() {
        isHelpVisible.toggle()
        if isHelpVisible {
            helpScrollOffset = 0
        }
    }

    public func closeHelpOrCancelCompose() -> Bool {
        if isHelpVisible {
            isHelpVisible = false
            return true
        }
        if isFlashVisible {
            cancelFlash()
            return true
        }
        if isComposing {
            isComposing = false
            activeRegion = .list
            return true
        }
        return false
    }

    public func scrollHelp(delta: Int) {
        helpScrollOffset = min(max(helpScrollOffset + delta, 0), 6)
    }

    public func scrollPreview(delta: Int) {
        let lineCount = selectedEntry?.preview.split(separator: "\n", omittingEmptySubsequences: false).count ?? 0
        previewScrollOffset = min(max(previewScrollOffset + delta, 0), max(lineCount - 1, 0))
    }

    public func handleKeyboardCommand(_ command: OverlayKeyboardCommand) -> OverlayKeyboardEffect {
        switch command {
        case .escape:
            return closeHelpOrCancelCompose() ? .none : .hideOverlay
        case .beginFlash(let focusOnMatch):
            guard !isHelpVisible, !isComposing else { return .passThrough }
            beginFlash(focusOnMatch: focusOnMatch)
            return .none
        case .tab(let reverse):
            guard !isHelpVisible else { return .none }
            cycleRegion(reverse: reverse)
            return .none
        case .moveUp:
            if isHelpVisible {
                scrollHelp(delta: -1)
            } else if activeRegion == .detail {
                scrollPreview(delta: -3)
            } else if !isComposing {
                moveSelection(delta: -1)
            } else {
                return .passThrough
            }
            return .none
        case .moveDown:
            if isHelpVisible {
                scrollHelp(delta: 1)
            } else if activeRegion == .detail {
                scrollPreview(delta: 3)
            } else if !isComposing {
                moveSelection(delta: 1)
            } else {
                return .passThrough
            }
            return .none
        case .pageUp:
            if isHelpVisible {
                scrollHelp(delta: -3)
            } else if activeRegion == .detail {
                scrollPreview(delta: -12)
            } else {
                moveSelection(delta: -8)
            }
            return .none
        case .pageDown:
            if isHelpVisible {
                scrollHelp(delta: 3)
            } else if activeRegion == .detail {
                scrollPreview(delta: 12)
            } else {
                moveSelection(delta: 8)
            }
            return .none
        case .enter(let command):
            if isComposing {
                return command ? .sendToSelected : .passThrough
            }
            if activeRegion == .pullRequest {
                return selectedEntry?.pullRequest == nil ? .focusSelected : .openPullRequest
            }
            return .focusSelected
        case .refresh:
            return .reload
        case .openSettings:
            return .openSettings
        case .cycleTheme:
            guard !isHelpVisible, !isComposing else { return .passThrough }
            cycleTheme()
            return .none
        case .deleteBackward:
            guard !isHelpVisible, !isComposing else { return .passThrough }
            if !query.isEmpty {
                query.removeLast()
            }
            return .none
        case .typed(let characters):
            if isFlashVisible {
                let filtered = characters.lowercased().filter { $0.isLetter }
                guard !filtered.isEmpty else { return .none }
                flashQuery.append(contentsOf: filtered)
                let matches = entries.compactMap { entry -> (AgentEntry, String)? in
                    guard let label = flashLabel(for: entry)?.lowercased(), label.hasPrefix(flashQuery) else { return nil }
                    return (entry, label)
                }
                if let exact = matches.first(where: { $0.1 == flashQuery }) {
                    selectionId = exact.0.id
                    let shouldFocus = flashFocusesOnMatch
                    cancelFlash()
                    return shouldFocus ? .focusSelected : .none
                }
                if matches.isEmpty {
                    cancelFlash()
                }
                return .none
            }
            if characters == "?" {
                toggleHelp()
                return .none
            }
            if isHelpVisible {
                if characters == "j" {
                    scrollHelp(delta: 1)
                    return .none
                }
                if characters == "k" {
                    scrollHelp(delta: -1)
                    return .none
                }
                return .passThrough
            }
            if activeRegion == .detail {
                if characters == "j" {
                    scrollPreview(delta: 3)
                    return .none
                }
                if characters == "k" {
                    scrollPreview(delta: -3)
                    return .none
                }
            }
            guard !isComposing else { return .passThrough }
            query.append(contentsOf: characters)
            return .none
        }
    }

    public func focusSelected() {
        guard let selectedEntry else { return }
        Task {
            do {
                try await client.focus(selectedEntry)
                appRouter?.overlayDidFocusPane()
            } catch {
                errorMessage = error.localizedDescription
            }
        }
    }

    public func requestOpenSettings() {
        appRouter?.overlayOpenSettings()
    }

    public func requestOpenURL(_ urlString: String) {
        appRouter?.overlayOpenURL(urlString)
    }

    public func openSelectedPullRequest() {
        guard let url = selectedEntry?.pullRequest?.url else { return }
        requestOpenURL(url)
    }

    public func sendToSelected() {
        guard let selectedEntry, !composeText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let text = composeText
        Task {
            do {
                try await client.send(selectedEntry, text: text)
                composeText = ""
                isComposing = false
                activeRegion = .list
                reload()
            } catch {
                errorMessage = error.localizedDescription
            }
        }
    }
}
