import AppKit

import ForemanOverlayCore

protocol TerminalActivating {
    func activateAfterFocus()
}

protocol PreviousApplicationActivating {
    @discardableResult
    func activatePreviousApplication() -> Bool
}

struct NoopPreviousApplicationActivator: PreviousApplicationActivating {
    func activatePreviousApplication() -> Bool { false }
}

final class RecentApplicationActivator: @unchecked Sendable, PreviousApplicationActivating {
    private let ignoredActivationBundleIdentifiers = [
        "dev.foreman.app",
        "com.raycast.macos",
        "com.apple.Spotlight",
    ]
    private var recentApplicationBundleIdentifier: String?
    private var observer: NSObjectProtocol?

    init() {
        let frontmost = NSWorkspace.shared.frontmostApplication?.bundleIdentifier
        if let frontmost, !ignoredActivationBundleIdentifiers.contains(frontmost) {
            recentApplicationBundleIdentifier = frontmost
        }
        observer = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let self,
                  let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication,
                  let bundleIdentifier = app.bundleIdentifier else { return }
            guard !ignoredActivationBundleIdentifiers.contains(bundleIdentifier) else { return }
            recentApplicationBundleIdentifier = bundleIdentifier
        }
    }

    deinit {
        if let observer {
            NSWorkspace.shared.notificationCenter.removeObserver(observer)
        }
    }

    func captureFrontmostApplication() {
        guard let bundleIdentifier = NSWorkspace.shared.frontmostApplication?.bundleIdentifier else { return }
        guard !ignoredActivationBundleIdentifiers.contains(bundleIdentifier) else { return }
        recentApplicationBundleIdentifier = bundleIdentifier
    }

    func activatePreviousApplication() -> Bool {
        guard let recentApplicationBundleIdentifier,
              let app = NSRunningApplication.runningApplications(withBundleIdentifier: recentApplicationBundleIdentifier).first else {
            NSLog("Foreman overlay previous app activation skipped; no previous app recorded")
            return false
        }
        if app.activate(options: [.activateAllWindows]) {
            NSLog("Foreman overlay activated previous app: \(recentApplicationBundleIdentifier)")
            return true
        }
        return false
    }
}

struct NoopTerminalActivator: TerminalActivating {
    func activateAfterFocus() {}
}

struct BundleTerminalActivator: TerminalActivating {
    let bundleIdentifier: String

    func activateAfterFocus() {
        guard Self.activate(bundleIdentifier: bundleIdentifier) else {
            NSLog("Foreman overlay terminal activation skipped; app not running: \(bundleIdentifier)")
            return
        }
    }

    @discardableResult
    static func activate(bundleIdentifier: String) -> Bool {
        guard let app = NSRunningApplication.runningApplications(withBundleIdentifier: bundleIdentifier).first else {
            return false
        }
        return app.activate(options: [.activateAllWindows])
    }
}

final class AutoTerminalActivator: @unchecked Sendable, TerminalActivating {
    private let terminalBundleIdentifiers = [
        "com.mitchellh.ghostty",
        "com.googlecode.iterm2",
        "com.apple.Terminal",
        "com.github.wez.wezterm",
        "org.alacritty",
        "net.kovidgoyal.kitty",
    ]
    private let ignoredActivationBundleIdentifiers = [
        "dev.foreman.app",
        "com.raycast.macos",
        "com.apple.Spotlight",
        "com.apple.finder",
    ]
    private var recentTerminalBundleIdentifier: String?
    private var observer: NSObjectProtocol?

    init() {
        recentTerminalBundleIdentifier = NSWorkspace.shared.frontmostApplication?.bundleIdentifier
        observer = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let self,
                  let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication,
                  let bundleIdentifier = app.bundleIdentifier else { return }
            guard !ignoredActivationBundleIdentifiers.contains(bundleIdentifier) else { return }
            if terminalBundleIdentifiers.contains(bundleIdentifier) {
                recentTerminalBundleIdentifier = bundleIdentifier
            }
        }
    }

    deinit {
        if let observer {
            NSWorkspace.shared.notificationCenter.removeObserver(observer)
        }
    }

    func activateAfterFocus() {
        if let recentTerminalBundleIdentifier,
           terminalBundleIdentifiers.contains(recentTerminalBundleIdentifier),
           BundleTerminalActivator.activate(bundleIdentifier: recentTerminalBundleIdentifier) {
            NSLog("Foreman overlay activated previous terminal: \(recentTerminalBundleIdentifier)")
            return
        }
        for bundleIdentifier in terminalBundleIdentifiers {
            if BundleTerminalActivator.activate(bundleIdentifier: bundleIdentifier) {
                NSLog("Foreman overlay activated running terminal: \(bundleIdentifier)")
                return
            }
        }
        NSLog("Foreman overlay terminal activation skipped; no known terminal app is running")
    }
}

struct TerminalActivatorFactory {
    @MainActor
    static func fromPreferences(_ preferences: OverlayPreferences) -> TerminalActivating {
        if let raw = ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_TERMINAL_ACTIVATION"], !raw.isEmpty {
            return from(preference: OverlayTerminalActivationPreference(raw: raw))
        }
        return from(preference: OverlayTerminalActivationPreference(
            raw: preferences.terminalActivation,
            customBundleID: preferences.customTerminalBundleID
        ))
    }

    static func fromEnvironment() -> TerminalActivating {
        from(preference: OverlayTerminalActivationPreference(raw: ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_TERMINAL_ACTIVATION"] ?? "auto"))
    }

    private static func from(preference: OverlayTerminalActivationPreference) -> TerminalActivating {
        switch preference {
        case .auto:
            return AutoTerminalActivator()
        case .none:
            return NoopTerminalActivator()
        case .bundle(let bundleIdentifier):
            return BundleTerminalActivator(bundleIdentifier: bundleIdentifier)
        }
    }
}
