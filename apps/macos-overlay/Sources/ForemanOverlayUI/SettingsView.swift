import SwiftUI
import KeyboardShortcuts

import ForemanOverlayCore

public extension KeyboardShortcuts.Name {
    static let toggleForemanOverlay = Self(
        "toggleForemanOverlay",
        default: .init(.f, modifiers: [.command, .option])
    )
}

private enum SettingsSection: String, CaseIterable, Identifiable {
    case general = "General"
    case view = "View"
    case focus = "Focus"
    case notifications = "Notifications"
    var id: String { rawValue }
}

public struct SettingsView: View {
    @ObservedObject var preferences: OverlayPreferences
    let hotkeyStatus: String?
    let onClearShortcut: () -> Void
    let onRestoreDefault: () -> Void
    let onShortcutChanged: () -> Void
    @State private var section: SettingsSection

    public init(
        preferences: OverlayPreferences,
        initialSection: String = "General",
        hotkeyStatus: String? = nil,
        onClearShortcut: @escaping () -> Void,
        onRestoreDefault: @escaping () -> Void,
        onShortcutChanged: @escaping () -> Void = {}
    ) {
        self.preferences = preferences
        self.hotkeyStatus = hotkeyStatus
        self.onClearShortcut = onClearShortcut
        self.onRestoreDefault = onRestoreDefault
        self.onShortcutChanged = onShortcutChanged
        _section = State(initialValue: SettingsSection(rawValue: initialSection) ?? .general)
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            header
            Picker("Section", selection: $section) {
                ForEach(SettingsSection.allCases) { section in
                    Text(section.rawValue).tag(section)
                }
            }
            .pickerStyle(.segmented)
            .labelsHidden()

            Group {
                switch section {
                case .general: general
                case .view: view
                case .focus: focus
                case .notifications: notifications
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .padding(24)
        .frame(width: 640, height: 520)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Foreman Settings")
                .font(.title2.weight(.semibold))
            Text("Tune the macOS control surface without changing Foreman’s tmux runtime.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var general: some View {
        VStack(alignment: .leading, spacing: 14) {
            settingsCard("Shortcut") {
                HStack {
                    Text("Toggle Foreman")
                    Spacer()
                    KeyboardShortcuts.Recorder("", name: .toggleForemanOverlay) { _ in
                        onShortcutChanged()
                    }
                    .labelsHidden()
                }
                HStack {
                    Button("Clear Shortcut") { onClearShortcut() }
                    Button("Restore Default") { onRestoreDefault() }
                    Text("Default: Cmd+Option+F")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Text("Click the shortcut field, then press a key combination. Clear removes the global shortcut; Restore Default sets Cmd+Option+F.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                if let hotkeyStatus {
                    Text(hotkeyStatus)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            settingsCard("Popup Size") {
                Picker("Size", selection: $preferences.sizePreset) {
                    ForEach(OverlaySizePreset.allCases) { preset in
                        Text("\(preset.label) — \(preset.dimensionsLabel)").tag(preset)
                    }
                }
                .pickerStyle(.radioGroup)
                Button("Reset Popup Size") { preferences.resetPopupSize() }
            }
        }
    }

    private var view: some View {
        settingsCard("Agent Visibility") {
            Toggle("Include non-agent panes", isOn: $preferences.includeAllPanes)
            pickerRow("Sort", selection: $preferences.sortMode, values: OverlaySortMode.allCases) { $0.label }
            pickerRow("Harness", selection: $preferences.harnessFilter, values: OverlayHarnessFilter.allCases) { $0.label }
            pickerRow("Status", selection: $preferences.statusFilter, values: OverlayStatusFilter.allCases) { $0.label }
            pickerRow("Row title", selection: $preferences.rowDisplayMode, values: OverlayRowDisplayMode.allCases) { $0.label }
            Text("Filters apply immediately. Smart row titles disambiguate panes that share the same workspace.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var focus: some View {
        settingsCard("Terminal Activation") {
            Picker("After focus", selection: $preferences.terminalActivation) {
                Text("Auto").tag("auto")
                Text("None").tag("none")
                Text("Ghostty").tag("ghostty")
                Text("iTerm2").tag("iterm")
                Text("Terminal.app").tag("terminal")
                Text("WezTerm").tag("wezterm")
                Text("Alacritty").tag("alacritty")
                Text("Kitty").tag("kitty")
                Text("Custom bundle id").tag("custom")
            }
            TextField("Custom bundle id", text: $preferences.customTerminalBundleID)
                .textFieldStyle(.roundedBorder)
                .disabled(preferences.terminalActivation != "custom")
            Text("Auto returns to the most recently active known terminal after Foreman focuses tmux.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var notifications: some View {
        settingsCard("Notifications") {
            Label("Runtime-managed for now", systemImage: "bell.badge")
                .font(.headline)
            Text("Foreman’s notification behavior is still configured by the runtime/config. The Mac app will expose delivery toggles once the control API has a stable notification settings contract.")
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
        }
    }

    private func settingsCard<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title)
                .font(.headline)
            content()
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 14))
        .overlay(RoundedRectangle(cornerRadius: 14).stroke(Color(nsColor: .separatorColor).opacity(0.35), lineWidth: 1))
    }

    private func pickerRow<Value: Hashable>(_ title: String, selection: Binding<Value>, values: [Value], label: @escaping (Value) -> String) -> some View {
        HStack {
            Text(title)
            Spacer()
            Picker(title, selection: selection) {
                ForEach(values, id: \.self) { value in
                    Text(label(value)).tag(value)
                }
            }
            .labelsHidden()
            .frame(width: 240)
        }
    }
}
