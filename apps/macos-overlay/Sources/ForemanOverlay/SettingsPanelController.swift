import AppKit
import Carbon.HIToolbox
import SwiftUI

import ForemanOverlayCore
import ForemanOverlayUI

final class SettingsPanel: NSPanel {
    override func keyDown(with event: NSEvent) {
        if Int(event.keyCode) == kVK_Escape {
            self.close()
            return
        }
        super.keyDown(with: event)
    }
}

@MainActor
final class SettingsPanelController {
    private var panel: NSPanel?
    private weak var preferences: OverlayPreferences?
    private var hotkeyStatus: String?
    private var onReset: (() -> Void)?
    private var onShortcutChanged: (() -> Void)?

    var isVisible: Bool {
        panel?.isVisible ?? false
    }

    func show(
        preferences: OverlayPreferences,
        hotkeyStatus: String?,
        onReset: @escaping () -> Void,
        onShortcutChanged: @escaping () -> Void
    ) {
        self.preferences = preferences
        self.hotkeyStatus = hotkeyStatus
        self.onReset = onReset
        self.onShortcutChanged = onShortcutChanged
        if panel == nil {
            panel = SettingsPanel(
                contentRect: NSRect(x: 0, y: 0, width: 640, height: 560),
                styleMask: [.titled, .closable],
                backing: .buffered,
                defer: false
            )
            panel?.title = "Foreman Settings"
            panel?.isReleasedWhenClosed = false
            panel?.level = .modalPanel
            panel?.isFloatingPanel = true
            panel?.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        }
        renderContent()
        panel?.center()
        NSApp.activate(ignoringOtherApps: true)
        panel?.orderFrontRegardless()
        panel?.makeKeyAndOrderFront(nil)
    }

    func updateHotkeyStatus(_ hotkeyStatus: String?) {
        self.hotkeyStatus = hotkeyStatus
        guard isVisible else { return }
        renderContent()
    }

    private func renderContent() {
        guard let preferences else { return }
        panel?.contentView = NSHostingView(rootView: SettingsView(
            preferences: preferences,
            hotkeyStatus: hotkeyStatus,
            onReset: { [weak self] in self?.onReset?() },
            onShortcutChanged: { [weak self] in self?.onShortcutChanged?() }
        ))
    }
}
