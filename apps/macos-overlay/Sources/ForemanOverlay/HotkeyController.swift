import AppKit
import Carbon.HIToolbox
import Foundation
import KeyboardShortcuts

enum HotkeySource: String {
    case environment = "environment override"
    case recorder = "settings recorder"
}

struct HotkeyStatus {
    let label: String
    let source: HotkeySource
    let isRegistered: Bool
    let message: String

    var settingsText: String {
        isRegistered ? "Registered via \(source.rawValue)" : "Not registered: \(message)"
    }
}

struct HotkeySpec {
    let keyCode: UInt32
    let modifiers: UInt32
    let label: String

    static let defaultSpec = HotkeySpec(keyCode: UInt32(kVK_ANSI_F), modifiers: UInt32(controlKey), label: "⌃F")

    static let userDefaultsKey = "ForemanOverlayHotkey"

    static func fromPreferences() -> HotkeySpec {
        if let raw = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_HOTKEY"], !raw.isEmpty {
            return parse(raw) ?? defaultSpec
        }
        if let raw = UserDefaults.standard.string(forKey: userDefaultsKey), !raw.isEmpty {
            return parse(raw) ?? defaultSpec
        }
        return defaultSpec
    }

    static func fromEnvironment() -> HotkeySpec {
        fromPreferences()
    }

    @MainActor
    static func fromShortcut(_ shortcut: KeyboardShortcuts.Shortcut) -> HotkeySpec {
        HotkeySpec(
            keyCode: UInt32(shortcut.carbonKeyCode),
            modifiers: UInt32(shortcut.carbonModifiers),
            label: shortcut.description
        )
    }

    static func parse(_ raw: String) -> HotkeySpec? {
        let parts = raw.lowercased().split(separator: "+").map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
        guard let key = parts.last, let keyCode = keyCodes[key] else { return nil }
        var modifiers: UInt32 = 0
        var labels: [String] = []
        for part in parts.dropLast() {
            switch part {
            case "cmd", "command", "⌘": modifiers |= UInt32(cmdKey); labels.append("⌘")
            case "opt", "option", "alt", "⌥": modifiers |= UInt32(optionKey); labels.append("⌥")
            case "ctrl", "control", "⌃": modifiers |= UInt32(controlKey); labels.append("⌃")
            case "shift", "⇧": modifiers |= UInt32(shiftKey); labels.append("⇧")
            default: return nil
            }
        }
        guard modifiers != 0 else { return nil }
        labels.append(String(key).uppercased())
        return HotkeySpec(keyCode: UInt32(keyCode), modifiers: modifiers, label: labels.joined())
    }

    private static let keyCodes: [String: Int] = [
        "a": kVK_ANSI_A, "b": kVK_ANSI_B, "c": kVK_ANSI_C, "d": kVK_ANSI_D,
        "e": kVK_ANSI_E, "f": kVK_ANSI_F, "g": kVK_ANSI_G, "h": kVK_ANSI_H,
        "i": kVK_ANSI_I, "j": kVK_ANSI_J, "k": kVK_ANSI_K, "l": kVK_ANSI_L,
        "m": kVK_ANSI_M, "n": kVK_ANSI_N, "o": kVK_ANSI_O, "p": kVK_ANSI_P,
        "q": kVK_ANSI_Q, "r": kVK_ANSI_R, "s": kVK_ANSI_S, "t": kVK_ANSI_T,
        "u": kVK_ANSI_U, "v": kVK_ANSI_V, "w": kVK_ANSI_W, "x": kVK_ANSI_X,
        "y": kVK_ANSI_Y, "z": kVK_ANSI_Z,
        "0": kVK_ANSI_0, "1": kVK_ANSI_1, "2": kVK_ANSI_2, "3": kVK_ANSI_3,
        "4": kVK_ANSI_4, "5": kVK_ANSI_5, "6": kVK_ANSI_6, "7": kVK_ANSI_7,
        "8": kVK_ANSI_8, "9": kVK_ANSI_9,
        "space": kVK_Space,
    ]
}

func fourCharCode(_ string: String) -> OSType {
    string.utf8.reduce(0) { ($0 << 8) + OSType($1) }
}

final class HotkeyController: @unchecked Sendable {
    private var hotKeyRef: EventHotKeyRef?
    private var eventHandler: EventHandlerRef?
    private var spec: HotkeySpec
    private var source: HotkeySource
    private let onPressed: @MainActor @Sendable () -> Void
    private(set) var status: HotkeyStatus

    init(spec: HotkeySpec = .fromEnvironment(), source: HotkeySource = .environment, onPressed: @escaping @MainActor @Sendable () -> Void) {
        self.spec = spec
        self.source = source
        self.onPressed = onPressed
        status = HotkeyStatus(label: spec.label, source: source, isRegistered: false, message: "registration pending")
        register()
    }

    deinit {
        unregister()
        removeEventHandler()
    }

    func update(spec: HotkeySpec, source: HotkeySource? = nil) {
        unregister()
        removeEventHandler()
        self.spec = spec
        if let source { self.source = source }
        status = HotkeyStatus(label: spec.label, source: self.source, isRegistered: false, message: "registration pending")
        register()
    }

    private func removeEventHandler() {
        if let eventHandler {
            RemoveEventHandler(eventHandler)
            self.eventHandler = nil
        }
    }

    private func unregister() {
        if let hotKeyRef {
            UnregisterEventHotKey(hotKeyRef)
            self.hotKeyRef = nil
        }
    }

    private func register() {
        guard let dispatcherTarget = GetEventDispatcherTarget() else {
            status = HotkeyStatus(label: spec.label, source: source, isRegistered: false, message: "Carbon event dispatcher target unavailable")
            NSLog("Foreman overlay hotkey registration failed: \(status.message)")
            return
        }
        let hotKeyID = EventHotKeyID(signature: fourCharCode("Frmn"), id: 1)
        let registerError = RegisterEventHotKey(spec.keyCode, spec.modifiers, hotKeyID, dispatcherTarget, 0, &hotKeyRef)
        guard registerError == noErr, hotKeyRef != nil else {
            status = HotkeyStatus(label: spec.label, source: source, isRegistered: false, message: "Carbon registration failed with OSStatus \(registerError)")
            NSLog("Foreman overlay hotkey registration failed: \(status.message)")
            return
        }

        var eventSpec = EventTypeSpec(eventClass: OSType(kEventClassKeyboard), eventKind: UInt32(kEventHotKeyPressed))
        let selfPointer = UnsafeMutableRawPointer(Unmanaged.passUnretained(self).toOpaque())
        let handlerError = InstallEventHandler(dispatcherTarget, { _, _, userData in
            guard let userData else { return noErr }
            let controller = Unmanaged<HotkeyController>.fromOpaque(userData).takeUnretainedValue()
            Task { @MainActor in controller.onPressed() }
            return noErr
        }, 1, &eventSpec, selfPointer, &eventHandler)
        guard handlerError == noErr else {
            unregister()
            status = HotkeyStatus(label: spec.label, source: source, isRegistered: false, message: "Carbon handler install failed with OSStatus \(handlerError)")
            NSLog("Foreman overlay hotkey handler installation failed: \(status.message)")
            return
        }
        status = HotkeyStatus(label: spec.label, source: source, isRegistered: true, message: "registered")
        NSLog("Foreman overlay hotkey registered from \(source.rawValue): \(spec.label)")
    }
}
