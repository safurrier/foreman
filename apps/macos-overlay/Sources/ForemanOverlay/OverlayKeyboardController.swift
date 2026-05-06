import AppKit
import Carbon.HIToolbox

import ForemanOverlayCore

enum OverlayKeyCommandAdapter {
    @MainActor
    static func shouldPassThroughTextEditing(_ event: NSEvent, store: OverlayStore) -> Bool {
        guard NSApp.keyWindow?.firstResponder is NSTextView else { return false }
        if store.isFlashVisible { return false }
        if store.isHelpVisible { return false }
        let activeRegion = store.activeRegion
        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        let hasCommand = modifiers.contains(.command)
        let hasControl = modifiers.contains(.control)
        let hasOption = modifiers.contains(.option)
        let hasOnlyTextEditingModifiers = !hasControl && modifiers.subtracting([.command, .option, .shift]).isEmpty

        if activeRegion == .compose {
            if Int(event.keyCode) == kVK_Return && hasCommand { return false }
            if Int(event.keyCode) == kVK_Escape || Int(event.keyCode) == kVK_Tab { return false }
            return true
        }

        switch Int(event.keyCode) {
        case kVK_UpArrow, kVK_DownArrow, kVK_PageUp, kVK_PageDown:
            return false
        case kVK_Delete, kVK_LeftArrow, kVK_RightArrow, kVK_Home, kVK_End:
            return hasOnlyTextEditingModifiers
        case kVK_ANSI_A where hasCommand:
            return true
        default:
            guard !hasCommand, !hasControl, !hasOption else { return false }
            guard let characters = event.characters, !characters.isEmpty else { return false }
            if activeRegion == .list && characters == "?" { return false }
            return characters.unicodeScalars.allSatisfy { scalar in
                !CharacterSet.controlCharacters.contains(scalar)
            }
        }
    }

    static func command(for event: NSEvent) -> OverlayKeyboardCommand? {
        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        let hasCommand = modifiers.contains(.command)
        let hasControl = modifiers.contains(.control)
        let hasOption = modifiers.contains(.option)

        switch Int(event.keyCode) {
        case kVK_Escape:
            return .escape
        case kVK_ANSI_J where hasCommand:
            return .beginFlash(focusOnMatch: modifiers.contains(.shift))
        case kVK_Tab:
            return .tab(reverse: modifiers.contains(.shift))
        case kVK_UpArrow:
            return .moveUp
        case kVK_DownArrow:
            return .moveDown
        case kVK_PageUp:
            return .pageUp
        case kVK_PageDown:
            return .pageDown
        case kVK_Return:
            return .enter(command: hasCommand)
        case kVK_ANSI_R where hasCommand:
            return .refresh
        case kVK_ANSI_T where hasCommand:
            return .cycleTheme
        case kVK_ANSI_Comma where hasCommand:
            return .openSettings
        case kVK_Delete where !hasCommand && !hasControl && !hasOption:
            return .deleteBackward
        default:
            guard !hasCommand, !hasControl, !hasOption else { return nil }
            guard let characters = event.characters, !characters.isEmpty else { return nil }
            guard characters.unicodeScalars.allSatisfy({ scalar in
                !CharacterSet.controlCharacters.contains(scalar)
            }) else { return nil }
            return .typed(characters)
        }
    }
}

@MainActor
enum OverlayKeyboardEffectExecutor {
    static func execute(_ effect: OverlayKeyboardEffect, store: OverlayStore, panelController: OverlayPanelController) -> NSEvent? {
        switch effect {
        case .none:
            return nil
        case .passThrough:
            return nil
        case .hideOverlay:
            panelController.hide()
            return nil
        case .focusSelected:
            store.focusSelected()
            return nil
        case .sendToSelected:
            store.sendToSelected()
            return nil
        case .openPullRequest:
            store.openSelectedPullRequest()
            return nil
        case .reload:
            store.reload()
            return nil
        case .openSettings:
            store.requestOpenSettings()
            return nil
        }
    }
}

@MainActor
final class OverlayKeyboardController {
    private weak var store: OverlayStore?
    private weak var panelController: OverlayPanelController?
    private var monitor: Any?

    init(store: OverlayStore, panelController: OverlayPanelController) {
        self.store = store
        self.panelController = panelController
        monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            self?.handle(event) ?? event
        }
    }

    private func handle(_ event: NSEvent) -> NSEvent? {
        guard let store, let panelController, panelController.isVisible, panelController.hasKeyWindow else { return event }
        if OverlayKeyCommandAdapter.shouldPassThroughTextEditing(event, store: store) {
            return event
        }
        guard let command = OverlayKeyCommandAdapter.command(for: event) else { return event }
        let effect = store.handleKeyboardCommand(command)
        if effect == .passThrough {
            return event
        }
        return OverlayKeyboardEffectExecutor.execute(effect, store: store, panelController: panelController)
    }
}
