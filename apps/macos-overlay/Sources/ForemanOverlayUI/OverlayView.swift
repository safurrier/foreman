import AppKit
import SwiftUI

import ForemanOverlayCore
public struct OverlayView: View {
    @ObservedObject private var store: OverlayStore
    private let autoReload: Bool

    public init(store: OverlayStore, autoReload: Bool = true) {
        self.store = store
        self.autoReload = autoReload
    }
    @FocusState private var searchFocused: Bool
    @FocusState private var composeFocused: Bool

    public var body: some View {
        ZStack {
            VStack(spacing: 0) {
                searchBar
                Divider()
                content
                Divider()
                footer
            }
            .allowsHitTesting(!store.isHelpVisible)
            if store.isHelpVisible {
                Color.black.opacity(0.18)
                    .contentShape(Rectangle())
                    .ignoresSafeArea()
                    .onTapGesture { }
                HelpOverlay(store: store)
                    .transition(.opacity.combined(with: .scale(scale: 0.98)))
                    .zIndex(1)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(themeBackground)
        .onAppear {
            searchFocused = true
            if autoReload {
                store.reload()
            }
        }
        .onChange(of: store.activeRegion) { _, region in
            searchFocused = region == .list && !store.isHelpVisible
            composeFocused = region == .compose
        }
        .onChange(of: store.isHelpVisible) { _, isVisible in
            searchFocused = !isVisible && store.activeRegion == .list
        }
    }

    private var searchBar: some View {
        HStack(spacing: 10) {
            Image(systemName: "magnifyingglass")
                .font(.system(size: 18, weight: .medium))
                .foregroundStyle(.secondary)
            TextField("Search Agent Sessions", text: $store.query)
                .textFieldStyle(.plain)
                .font(.system(size: 20, weight: .medium))
                .focused($searchFocused)
                .onSubmit { store.focusSelected() }
            if store.isLoading {
                ProgressView().scaleEffect(0.65)
            }
        }
        .frame(height: 58)
        .padding(.horizontal, 16)
    }

    @ViewBuilder
    private var content: some View {
        if let errorMessage = store.errorMessage {
            VStack(spacing: 12) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .font(.largeTitle)
                    .foregroundStyle(.orange)
                Text("Foreman overlay could not load agents")
                    .font(.headline)
                Text(errorMessage)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                Text("Try: foreman --doctor")
                    .font(.caption.monospaced())
                    .padding(8)
                    .background(.quaternary, in: RoundedRectangle(cornerRadius: 8))
                Text("If this app was launched from Raycast/Spotlight, Foreman uses its bundled control binary and the current tmux environment.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .padding()
        } else if store.response == nil && store.isLoading {
            loadingState
        } else if store.entries.isEmpty && !store.isLoading {
            emptyState
        } else {
            HStack(spacing: 0) {
                VStack(spacing: 0) {
                    sourceDiagnosticsBanner
                    agentList
                }
                .frame(width: 360)
                Divider()
                detailPane
            }
        }
    }

    private var loadingState: some View {
        VStack(spacing: 14) {
            Image(systemName: "terminal")
                .font(.system(size: 38, weight: .semibold))
                .foregroundStyle(themeAccent)
            Text("Foreman")
                .font(.title2.weight(.semibold))
            HStack(spacing: 8) {
                ProgressView().scaleEffect(0.7)
                Text("Finding agent sessions…")
                    .font(.headline)
            }
            Text("Checking tmux panes, pull requests, and extension providers.")
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
    }

    @ViewBuilder
    private var emptyState: some View {
        if let response = store.response, !response.entries.isEmpty {
            VStack(spacing: 10) {
                ContentUnavailableView("No matching agent sessions", systemImage: "magnifyingglass", description: Text("No sessions match “\(store.query)”."))
                Button("Clear Search") { store.query = "" }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let diagnostic = store.response?.allDiagnostics.first {
            VStack(spacing: 10) {
                ContentUnavailableView("Foreman control diagnostic", systemImage: "exclamationmark.triangle", description: Text(diagnostic.message))
                Text("Try: foreman --doctor")
                    .font(.caption.monospaced())
                    .padding(8)
                    .background(.quaternary, in: RoundedRectangle(cornerRadius: 8))
                Text("Foreman returned no agent panes and reported a \(diagnostic.level) diagnostic. Fix the local tmux/control environment, then Refresh.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .frame(maxWidth: 520)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            VStack(spacing: 10) {
                ContentUnavailableView("No Foreman agents", systemImage: "terminal", description: Text("No agent panes matched Foreman's agent detector."))
                Text("Try Refresh, start an agent in tmux, or launch with FOREMAN_OVERLAY_ALL_PANES=1 to include non-agent panes while debugging.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .frame(maxWidth: 520)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    @ViewBuilder
    private var sourceDiagnosticsBanner: some View {
        if let diagnostic = store.response?.sourceDiagnostics.first {
            HStack(alignment: .top, spacing: 8) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.orange)
                VStack(alignment: .leading, spacing: 2) {
                    Text(diagnostic.sourceLabel ?? "Source diagnostic")
                        .font(.caption.weight(.semibold))
                    Text(diagnostic.message)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)
                }
                Spacer(minLength: 0)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(.orange.opacity(0.10))
            Divider()
        }
    }

    private var agentList: some View {
        ScrollViewReader { proxy in
            List(store.presentedEntries, selection: $store.selectionId) { row in
                AgentRow(
                    entry: row.entry,
                    presentation: row.presentation,
                    flashLabel: store.flashLabel(for: row.entry),
                    flashQuery: store.flashQuery
                )
                    .tag(row.entry.id)
                    .id(row.entry.id)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        store.selectionId = row.entry.id
                        store.activateRegion(.list)
                    }
                    .onTapGesture(count: 2) {
                        store.selectionId = row.entry.id
                        store.focusSelected()
                    }
            }
            .listStyle(.sidebar)
            .scrollDisabled(store.isHelpVisible)
            .activePanelBorder(isActive: store.activeRegion == .list && !store.isHelpVisible, accent: themeAccent)
            .accessibilityLabel(store.activeRegion == .list ? "Active region: Agent sessions" : "Agent sessions")
            .onChange(of: store.selectionId) { _, id in
                guard let id else { return }
                withAnimation(.easeInOut(duration: 0.12)) {
                    proxy.scrollTo(id, anchor: .center)
                }
            }
            .onMoveCommand { direction in
                switch direction {
                case .up: store.moveSelection(delta: -1)
                case .down: store.moveSelection(delta: 1)
                default: break
                }
            }
        }
    }

    @ViewBuilder
    private var detailPane: some View {
        if let entry = store.selectedEntry {
            VStack(alignment: .leading, spacing: 12) {
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(entry.navigationTitle)
                            .font(.title3.weight(.semibold))
                        Text("\(entry.sourceLabel) / \(entry.sessionName) / \(entry.windowName) / \(entry.paneId)")
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                    StatusBadge(status: entry.status, label: entry.statusLabel)
                }

                metadata(entry)

                if let pullRequest = entry.pullRequest {
                    PullRequestCardView(pullRequest: pullRequest, accent: themeAccent) { store.requestOpenURL(pullRequest.url) }
                        .activePanelBorder(isActive: store.activeRegion == .pullRequest && !store.isHelpVisible, accent: themeAccent)
                        .contentShape(Rectangle())
                        .onTapGesture { store.activateRegion(.pullRequest) }
                        .accessibilityLabel(store.activeRegion == .pullRequest ? "Active region: Pull Request" : "Pull Request")
                }

                ForEach(entry.extensionCards) { card in
                    ExtensionCardView(card: card, accent: themeAccent) { action in
                        performExtensionAction(action)
                    }
                }

                if entry.extensionCards.isEmpty && store.selectedEntryIsLoadingExtensions {
                    Text("Extension cards loading…")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                } else if entry.extensionCards.isEmpty, let extensionError = store.selectedEntryExtensionError {
                    Text("Extension cards unavailable: \(extensionError)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Text("Recent Output")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                PreviewOutputView(entry: entry, store: store)
                    .activePanelBorder(isActive: store.activeRegion == .detail && !store.isHelpVisible, accent: themeAccent)

                if store.isComposing {
                    composeInput(entry)
                }
            }
            .padding(16)
        }
    }

    private func metadata(_ entry: AgentEntry) -> some View {
        Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 6) {
            GridRow { metaLabel("Source"); Text(entry.sourceLabel) }
            GridRow { metaLabel("Harness"); Text(entry.harnessLabel ?? "Unknown") }
            GridRow { metaLabel("Status src"); Text(entry.statusSource ?? "Unknown") }
            GridRow { metaLabel("Command"); Text(entry.runtimeCommand ?? entry.currentCommand ?? "Unknown") }
            GridRow { metaLabel("Workspace"); Text(entry.workingDir ?? "Unknown").lineLimit(2) }
            if let linkedRepository = entry.linkedRepository {
                GridRow { metaLabel("Linked repo"); Text(linkedRepository).lineLimit(2) }
            }
        }
        .font(.caption)
    }

    private func metaLabel(_ text: String) -> some View {
        Text(text).foregroundStyle(.secondary).frame(width: 72, alignment: .trailing)
    }

    private func performExtensionAction(_ action: ControlExtensionAction) {
        switch action.kind {
        case "open-file":
            store.requestOpenURL(URL(fileURLWithPath: action.value).absoluteString)
        case "open-url":
            store.requestOpenURL(action.value)
        case "copy":
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(action.value, forType: .string)
        default:
            break
        }
    }

    private var footer: some View {
        HStack(spacing: 14) {
            ActiveRegionChip(region: store.activeRegion)
            if let flashPrompt = store.flashPrompt {
                FlashPromptChip(prompt: flashPrompt, focuses: store.flashFocusesOnMatch)
            }
            Divider().frame(height: 16)
            Button("Focus") { store.focusSelected() }
            Button("Refresh") { store.reload() }
            Button(store.isComposing ? "Send" : "Compose") {
                if store.isComposing {
                    store.sendToSelected()
                } else {
                    store.activateRegion(.compose)
                }
            }
            .disabled(store.selectedEntry == nil)
            Button("Settings") { store.requestOpenSettings() }
            Spacer()
            Text(store.footerHint)
                .foregroundStyle(.secondary)
            if let summary = store.lastReloadSummary {
                Text(summary)
                    .foregroundStyle(.secondary.opacity(0.8))
            }
            if let filterSummary = store.preferences.activeFilterSummary {
                Text(filterSummary)
                    .foregroundStyle(themeAccent)
            }
            Text("Theme: \(store.theme.label)")
                .foregroundStyle(themeAccent)
        }
        .font(.caption)
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
    }

    private func composeInput(_ entry: AgentEntry) -> some View {
        HStack(spacing: 8) {
            TextField("Send to \(entry.navigationTitle)", text: $store.composeText, axis: .vertical)
                .lineLimit(2...5)
                .focused($composeFocused)
                .onSubmit { store.sendToSelected() }
                .textFieldStyle(.plain)
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
            Button("Send") { store.sendToSelected() }
                .keyboardShortcut(.return, modifiers: .command)
                .disabled(store.composeText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
        }
        .padding(4)
        .background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 12))
        .activePanelBorder(isActive: store.activeRegion == .compose && !store.isHelpVisible, accent: themeAccent)
        .accessibilityLabel("Compose message")
        .onAppear { composeFocused = true }
    }


    private var themeAccent: Color {
        switch store.theme {
        case .system: .blue
        case .graphite: .gray
        case .indigo: .indigo
        case .terminal: .green
        }
    }

    @ViewBuilder
    private var themeBackground: some View {
        switch store.theme {
        case .system:
            Rectangle().fill(.regularMaterial)
        case .graphite:
            LinearGradient(colors: [Color(nsColor: .windowBackgroundColor), .gray.opacity(0.12)], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .indigo:
            LinearGradient(colors: [.indigo.opacity(0.20), Color(nsColor: .windowBackgroundColor)], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .terminal:
            LinearGradient(colors: [.black.opacity(0.18), .green.opacity(0.10), Color(nsColor: .windowBackgroundColor)], startPoint: .topLeading, endPoint: .bottomTrailing)
        }
    }

    private func shortcut(_ key: String, _ label: String) -> some View {
        HStack(spacing: 4) {
            Text(key).font(.caption.monospaced().weight(.semibold))
            Text(label).foregroundStyle(.secondary)
        }
    }
}
