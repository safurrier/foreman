import AppKit
import Carbon.HIToolbox
import SwiftUI

import ForemanOverlayCore
import ForemanOverlayUI

@MainActor
final class ForemanOverlayPanel: NSPanel {
    var store: OverlayStore?
    weak var panelController: OverlayPanelController?

    override func performKeyEquivalent(with event: NSEvent) -> Bool {
        guard let store, let panelController else { return super.performKeyEquivalent(with: event) }
        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        guard modifiers.contains(.command), let command = OverlayKeyCommandAdapter.command(for: event) else {
            return super.performKeyEquivalent(with: event)
        }
        let effect = store.handleKeyboardCommand(command)
        guard effect != .passThrough else { return super.performKeyEquivalent(with: event) }
        _ = OverlayKeyboardEffectExecutor.execute(effect, store: store, panelController: panelController)
        return true
    }

    override func noResponder(for eventSelector: Selector) {
        // Suppress AppKit's invalid-command beep for overlay keyboard-control
        // paths we intentionally handle at the panel/store layer.
        if eventSelector == #selector(NSResponder.keyDown(with:)) {
            return
        }
        super.noResponder(for: eventSelector)
    }
}

@MainActor
final class OverlayPanelController {
    private let panel: ForemanOverlayPanel
    private let store: OverlayStore
    private let preferences: OverlayPreferences
    private let previousApplicationActivator: PreviousApplicationActivating
    private let recentApplicationActivator: RecentApplicationActivator?

    init(store: OverlayStore, preferences: OverlayPreferences, previousApplicationActivator: PreviousApplicationActivating = NoopPreviousApplicationActivator()) {
        self.store = store
        self.preferences = preferences
        self.previousApplicationActivator = previousApplicationActivator
        self.recentApplicationActivator = previousApplicationActivator as? RecentApplicationActivator
        panel = ForemanOverlayPanel(
            contentRect: NSRect(x: 0, y: 0, width: preferences.popupWidth, height: preferences.popupHeight),
            styleMask: [.titled, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        panel.store = store
        panel.panelController = self
        panel.title = "Foreman"
        panel.titleVisibility = .hidden
        panel.titlebarAppearsTransparent = true
        panel.isMovableByWindowBackground = true
        panel.minSize = NSSize(width: OverlayPreferences.minPopupWidth, height: OverlayPreferences.minPopupHeight)
        panel.maxSize = NSSize(width: OverlayPreferences.maxPopupWidth, height: OverlayPreferences.maxPopupHeight)
        panel.level = .floating
        if ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_JOIN_ALL_SPACES"] == "0" {
            panel.collectionBehavior = [.fullScreenAuxiliary]
        } else {
            panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        }
        panel.contentView = NSHostingView(rootView: OverlayView(store: store, autoReload: false))
    }

    func toggle() {
        if panel.isVisible {
            hide()
        } else {
            show()
        }
    }

    func show() {
        recentApplicationActivator?.captureFrontmostApplication()
        store.reload()
        applyPreferredSize()
        positionOnActiveScreen()
        if ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_ACTIVATE"] == "0" {
            panel.orderFrontRegardless()
        } else {
            NSApp.activate(ignoringOtherApps: true)
            panel.makeKeyAndOrderFront(nil)
        }
        writeWindowNumberIfRequested()
        writePanelSnapshotIfRequested()
    }

    func hide(restorePreviousApplication: Bool = true) {
        persistCurrentSize()
        panel.orderOut(nil)
        if restorePreviousApplication && !previousApplicationActivator.activatePreviousApplication() {
            NSApp.hide(nil)
        }
    }

    private func applyPreferredSize() {
        panel.setContentSize(NSSize(width: preferences.popupWidth, height: preferences.popupHeight))
    }

    private func persistCurrentSize() {
        preferences.popupWidth = panel.frame.width
        preferences.popupHeight = panel.frame.height
    }

    func postDoubleClickFirstRowForTesting() {
        let point = NSPoint(x: 170, y: 440)
        for clickCount in [1, 2] {
            guard let down = NSEvent.mouseEvent(
                with: .leftMouseDown,
                location: point,
                modifierFlags: [],
                timestamp: ProcessInfo.processInfo.systemUptime,
                windowNumber: panel.windowNumber,
                context: nil,
                eventNumber: 0,
                clickCount: clickCount,
                pressure: 1
            ), let up = NSEvent.mouseEvent(
                with: .leftMouseUp,
                location: point,
                modifierFlags: [],
                timestamp: ProcessInfo.processInfo.systemUptime,
                windowNumber: panel.windowNumber,
                context: nil,
                eventNumber: 0,
                clickCount: clickCount,
                pressure: 0
            ) else { continue }
            NSApp.postEvent(down, atStart: false)
            NSApp.postEvent(up, atStart: false)
        }
    }

    var isVisible: Bool {
        panel.isVisible
    }

    var hasKeyWindow: Bool {
        panel.isKeyWindow
    }

    var windowNumber: Int {
        panel.windowNumber
    }

    private func positionOnActiveScreen() {
        let requestedScreen = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_SCREEN_INDEX"]
            .flatMap(Int.init)
            .flatMap { index in NSScreen.screens.indices.contains(index) ? NSScreen.screens[index] : nil }
        let screen = requestedScreen ?? NSScreen.main ?? NSScreen.screens.first
        guard let frame = screen?.visibleFrame else { return }
        let size = panel.frame.size
        let x = frame.midX - size.width / 2
        let y = frame.maxY - size.height - 90
        panel.setFrameOrigin(NSPoint(x: x, y: y))
    }

    private func writeWindowNumberIfRequested() {
        guard let path = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_WINDOW_ID_PATH"] else { return }
        try? "\(panel.windowNumber)\n".write(toFile: path, atomically: true, encoding: .utf8)
    }

    private func writePanelSnapshotIfRequested() {
        guard let path = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_PANEL_SNAPSHOT_PATH"] else { return }
        Task { @MainActor in
            try? await Task.sleep(for: .milliseconds(500))
            guard let view = panel.contentView else { return }
            view.layoutSubtreeIfNeeded()
            guard let representation = view.bitmapImageRepForCachingDisplay(in: view.bounds) else { return }
            representation.size = view.bounds.size
            view.cacheDisplay(in: view.bounds, to: representation)
            guard let png = representation.representation(using: .png, properties: [:]) else { return }
            try? png.write(to: URL(fileURLWithPath: path))
        }
    }
}
