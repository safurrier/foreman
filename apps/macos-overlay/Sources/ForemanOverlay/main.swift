import AppKit
import Carbon.HIToolbox
import Combine
import KeyboardShortcuts

import ForemanOverlayCore
import ForemanOverlayUI

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var refreshMenuItem: NSMenuItem?
    var preferences: OverlayPreferences!
    var store: OverlayStore!
    var panelController: OverlayPanelController!
    private var hotkeyController: HotkeyController?
    private var keyboardController: OverlayKeyboardController!
    private var settingsPanelController = SettingsPanelController()
    private var terminalActivator: TerminalActivating = NoopTerminalActivator()
    private let previousApplicationActivator = RecentApplicationActivator()
    private var preferenceCancellable: AnyCancellable?
    private var hotkeyStatusText = "Hotkey registration pending"

    private var isBundledApp: Bool {
        Bundle.main.bundlePath.hasSuffix(".app")
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        configureActivationPolicy()
        configureMainMenu()

        let foremanPath = resolvedForemanPath()
        preferences = OverlayPreferences()
        if ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_ALL_PANES"] == "1" {
            preferences.includeAllPanes = true
        }
        store = OverlayStore(client: ForemanClient(foremanPath: foremanPath, includeAllPanes: preferences.includeAllPanes), preferences: preferences)
        store.appRouter = self
        panelController = OverlayPanelController(
            store: store,
            preferences: preferences,
            previousApplicationActivator: previousApplicationActivator
        )
        terminalActivator = TerminalActivatorFactory.fromPreferences(preferences)
        preferenceCancellable = preferences.objectWillChange.sink { [weak self] _ in
            Task { @MainActor in
                try? await Task.sleep(for: .milliseconds(25))
                guard let self, let preferences = self.preferences else { return }
                self.terminalActivator = TerminalActivatorFactory.fromPreferences(preferences)
            }
        }
        keyboardController = OverlayKeyboardController(store: store, panelController: panelController)
        configureHotkey()

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        statusItem?.button?.title = hotkeyController?.status.label ?? HotkeySpec.defaultSpec.label
        statusItem?.button?.toolTip = hotkeyStatusText
        statusItem?.menu = makeStatusMenu()

        let showOnLaunch = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_SHOW_ON_LAUNCH"]
        if showOnLaunch == "1" || (showOnLaunch != "0" && isBundledApp) {
            panelController.show()
        }
        if ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_SHOW_SETTINGS_ON_LAUNCH"] == "1" {
            openSettings()
        }
        writeReadyFileIfRequested()
        runScriptedGauntletIfRequested()
    }

    func applicationDidBecomeActive(_ notification: Notification) {
        guard shouldRestorePanelOnActivation() else { return }
        panelController.show()
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        panelController?.show()
        return true
    }

    private func shouldRestorePanelOnActivation() -> Bool {
        guard isBundledApp else { return false }
        guard ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_RESTORE_ON_ACTIVATE"] != "0" else { return false }
        guard !panelController.isVisible else { return false }
        guard !settingsPanelController.isVisible else { return false }
        guard ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_EXIT_AFTER_READY"] != "1" else { return false }
        return true
    }

    private func configureActivationPolicy() {
        let requested = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_ACTIVATION_POLICY"]?.lowercased()
        switch requested {
        case "accessory", "agent", "menu-bar":
            NSApp.setActivationPolicy(.accessory)
        case "regular":
            NSApp.setActivationPolicy(.regular)
        default:
            NSApp.setActivationPolicy(isBundledApp ? .regular : .accessory)
        }
    }

    private func configureHotkey() {
        let spec: HotkeySpec
        let source: HotkeySource
        if let envHotkey = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_HOTKEY"], let hotkeySpec = HotkeySpec.parse(envHotkey) {
            spec = hotkeySpec
            source = .environment
        } else {
            let shortcutDefaultsKey = "KeyboardShortcuts_toggleForemanOverlay"
            if UserDefaults.standard.object(forKey: shortcutDefaultsKey) == nil {
                KeyboardShortcuts.reset(.toggleForemanOverlay)
            }
            spec = KeyboardShortcuts.getShortcut(for: .toggleForemanOverlay).map(HotkeySpec.fromShortcut) ?? HotkeySpec.defaultSpec
            source = .recorder
        }

        if let hotkeyController {
            hotkeyController.update(spec: spec, source: source)
        } else {
            hotkeyController = HotkeyController(spec: spec, source: source) { [weak self] in
                Task { @MainActor in self?.panelController.toggle() }
            }
        }
        refreshHotkeyStatusUI()
    }

    private func refreshHotkeyStatusUI() {
        hotkeyStatusText = hotkeyController?.status.settingsText ?? "Hotkey registration unknown"
        statusItem?.button?.title = hotkeyController?.status.label ?? HotkeySpec.defaultSpec.label
        statusItem?.button?.toolTip = hotkeyStatusText
        settingsPanelController.updateHotkeyStatus(hotkeyStatusText)
    }

    private func resolvedForemanPath() -> String {
        if let path = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_FOREMAN_PATH"], !path.isEmpty {
            return path
        }
        if let bundled = Bundle.main.url(forResource: "foreman", withExtension: nil) {
            return bundled.path
        }
        return "foreman"
    }

    private func writeReadyFileIfRequested() {
        guard let path = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_READY_FILE"] else { return }
        try? "ready\n".write(toFile: path, atomically: true, encoding: .utf8)
        if ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_EXIT_AFTER_READY"] == "1" {
            Task { @MainActor in
                try? await Task.sleep(for: .milliseconds(250))
                NSApp.terminate(nil)
            }
        }
    }

    @objc func showAbout() {
        let version = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "development"
        NSApp.orderFrontStandardAboutPanel(options: [
            .applicationName: "Foreman",
            .applicationVersion: version,
            .credits: NSAttributedString(string: "A native macOS control surface for Foreman tmux agents."),
        ])
    }

    @objc func openOverlay() { panelController.show() }
    @objc func refresh() { store.reload() }
    @objc func beginFlashJump() { store.beginFlash() }
    @objc func cycleTheme() { store.cycleTheme() }
    func clearShortcut() {
        KeyboardShortcuts.setShortcut(nil, for: .toggleForemanOverlay)
        configureHotkey()
    }

    func restoreDefaultShortcut() {
        KeyboardShortcuts.reset(.toggleForemanOverlay)
        configureHotkey()
    }

    @objc func openSettings() {
        settingsPanelController.show(
            preferences: preferences,
            hotkeyStatus: hotkeyStatusText,
            onClearShortcut: { [weak self] in self?.clearShortcut() },
            onRestoreDefault: { [weak self] in self?.restoreDefaultShortcut() },
            onShortcutChanged: { [weak self] in
                self?.configureHotkey()
            }
        )
    }
    @objc func quit() { NSApp.terminate(nil) }
}

extension AppDelegate: OverlayAppRouting {
    func overlayDidFocusPane() {
        panelController.hide(restorePreviousApplication: false)
        terminalActivator.activateAfterFocus()
    }

    func overlayOpenSettings() {
        openSettings()
    }

    func overlayOpenURL(_ urlString: String) {
        guard let url = URL(string: urlString) else { return }
        NSWorkspace.shared.open(url)
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.run()
