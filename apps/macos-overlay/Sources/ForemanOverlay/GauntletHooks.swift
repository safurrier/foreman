import AppKit
import Carbon.HIToolbox
import Darwin
import KeyboardShortcuts

extension AppDelegate {
    func runScriptedGauntletIfRequested() {
        guard ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_SCRIPTED_GAUNTLET"] == "1" else { return }
        Task { @MainActor in
            try? await Task.sleep(for: .milliseconds(700))
            store.query = ""
            guard let flashTarget = store.entries.dropFirst().first else {
                NSLog("overlay gauntlet flash failed; no second entry")
                exit(70)
            }
            postKey(keyCode: UInt16(kVK_ANSI_J), characters: "j", modifiers: .command)
            try? await Task.sleep(for: .milliseconds(150))
            guard let flashLabel = store.flashLabel(for: flashTarget)?.lowercased(),
                  let flashCharacter = flashLabel.first else {
                NSLog("overlay gauntlet flash failed; no label for target")
                exit(70)
            }
            postKey(keyCode: keyCode(for: flashCharacter), characters: String(flashCharacter))
            try? await Task.sleep(for: .milliseconds(150))
            guard store.selectedEntry?.paneId == flashTarget.paneId else {
                NSLog("overlay gauntlet flash failed; selected pane: \(store.selectedEntry?.paneId ?? "nil") expected: \(flashTarget.paneId)")
                exit(70)
            }
            NSLog("overlay gauntlet flash passed")
            postKey(keyCode: UInt16(kVK_ANSI_Slash), characters: "?")
            try? await Task.sleep(for: .milliseconds(150))
            guard store.isHelpVisible else {
                NSLog("overlay gauntlet help failed; help not visible")
                exit(71)
            }
            NSLog("overlay gauntlet help passed")
            let beforeHelpScroll = store.helpScrollOffset
            postKey(keyCode: UInt16(kVK_ANSI_J), characters: "j")
            try? await Task.sleep(for: .milliseconds(150))
            guard store.helpScrollOffset > beforeHelpScroll else {
                NSLog("overlay gauntlet help scroll failed; offset: \(store.helpScrollOffset)")
                exit(72)
            }
            NSLog("overlay gauntlet help scroll passed")
            _ = store.closeHelpOrCancelCompose()
            store.query = ""
            store.selectionId = store.entries.first?.id
            let arrowTarget = store.entries.dropFirst().first?.paneId
            postKey(keyCode: UInt16(kVK_DownArrow), characters: "\u{F701}")
            try? await Task.sleep(for: .milliseconds(150))
            guard store.selectedEntry?.paneId == arrowTarget else {
                NSLog("overlay gauntlet arrow navigation failed; selected pane: \(store.selectedEntry?.paneId ?? "nil") expected: \(arrowTarget ?? "nil")")
                exit(73)
            }
            NSLog("overlay gauntlet arrow navigation passed")
            store.selectionId = store.entries.first?.id
            try? await Task.sleep(for: .milliseconds(150))
            panelController.postDoubleClickFirstRowForTesting()
            try? await Task.sleep(for: .milliseconds(250))
            store.selectionId = store.entries.first?.id
            NSLog("overlay gauntlet focusing pane: \(store.selectedEntry?.paneId ?? "nil")")
            store.focusSelected()
            try? await Task.sleep(for: .milliseconds(450))
            panelController.show()
            try? await Task.sleep(for: .milliseconds(250))
            store.activateRegion(.detail)
            postKey(keyCode: UInt16(kVK_Return), characters: "\r")
            try? await Task.sleep(for: .milliseconds(450))
            panelController.show()
            try? await Task.sleep(for: .milliseconds(250))
            store.activateRegion(.compose)
            store.composeText = "overlay gauntlet send"
            try? await Task.sleep(for: .milliseconds(250))
            store.sendToSelected()
            try? await Task.sleep(for: .milliseconds(700))
            postKey(keyCode: UInt16(kVK_ANSI_R), characters: "r", modifiers: .command)
            try? await Task.sleep(for: .milliseconds(500))
            runShortcutSettingsGauntlet()
            NSApp.terminate(nil)
        }
    }

    func runShortcutSettingsGauntlet() {
        let defaultsKey = "KeyboardShortcuts_toggleForemanOverlay"
        let previousShortcut = UserDefaults.standard.object(forKey: defaultsKey)
        defer {
            if let previousShortcut {
                UserDefaults.standard.set(previousShortcut, forKey: defaultsKey)
            } else {
                UserDefaults.standard.removeObject(forKey: defaultsKey)
            }
        }

        openSettings()
        clearShortcut()
        guard KeyboardShortcuts.getShortcut(for: .toggleForemanOverlay) == nil else {
            NSLog("overlay gauntlet shortcut clear failed")
            exit(74)
        }
        restoreDefaultShortcut()
        guard let shortcut = KeyboardShortcuts.getShortcut(for: .toggleForemanOverlay),
              shortcut.carbonKeyCode == Int(HotkeySpec.defaultSpec.keyCode),
              shortcut.carbonModifiers == Int(HotkeySpec.defaultSpec.modifiers),
              KeyboardShortcuts.isEnabled(for: .toggleForemanOverlay) else {
            NSLog("overlay gauntlet shortcut restore failed")
            exit(75)
        }
        NSLog("overlay gauntlet shortcut settings passed")
    }

    func keyCode(for character: Character) -> UInt16 {
        switch character {
        case "a": UInt16(kVK_ANSI_A)
        case "b": UInt16(kVK_ANSI_B)
        case "c": UInt16(kVK_ANSI_C)
        case "d": UInt16(kVK_ANSI_D)
        case "e": UInt16(kVK_ANSI_E)
        case "f": UInt16(kVK_ANSI_F)
        case "g": UInt16(kVK_ANSI_G)
        case "h": UInt16(kVK_ANSI_H)
        case "i": UInt16(kVK_ANSI_I)
        case "j": UInt16(kVK_ANSI_J)
        case "k": UInt16(kVK_ANSI_K)
        case "l": UInt16(kVK_ANSI_L)
        case "m": UInt16(kVK_ANSI_M)
        case "n": UInt16(kVK_ANSI_N)
        case "o": UInt16(kVK_ANSI_O)
        case "p": UInt16(kVK_ANSI_P)
        case "q": UInt16(kVK_ANSI_Q)
        case "r": UInt16(kVK_ANSI_R)
        case "s": UInt16(kVK_ANSI_S)
        case "t": UInt16(kVK_ANSI_T)
        case "u": UInt16(kVK_ANSI_U)
        case "v": UInt16(kVK_ANSI_V)
        case "w": UInt16(kVK_ANSI_W)
        case "x": UInt16(kVK_ANSI_X)
        case "y": UInt16(kVK_ANSI_Y)
        case "z": UInt16(kVK_ANSI_Z)
        case " ": UInt16(kVK_Space)
        case "?": UInt16(kVK_ANSI_Slash)
        default: 0
        }
    }

    func postKey(keyCode: UInt16, characters: String, modifiers: NSEvent.ModifierFlags = []) {
        guard let event = NSEvent.keyEvent(
            with: .keyDown,
            location: .zero,
            modifierFlags: modifiers,
            timestamp: ProcessInfo.processInfo.systemUptime,
            windowNumber: panelController?.windowNumber ?? 0,
            context: nil,
            characters: characters,
            charactersIgnoringModifiers: characters,
            isARepeat: false,
            keyCode: keyCode
        ) else { return }
        NSApp.postEvent(event, atStart: false)
    }

}
