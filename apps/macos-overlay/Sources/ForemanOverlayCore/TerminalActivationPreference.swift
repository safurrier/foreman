import Foundation

public enum OverlayTerminalActivationPreference: Equatable, Sendable {
    case auto
    case none
    case bundle(String)

    public init(raw: String, customBundleID: String = "") {
        switch raw.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() {
        case "auto", "1", "true", "":
            self = .auto
        case "none", "0", "false":
            self = .none
        case "terminal":
            self = .bundle("com.apple.Terminal")
        case "iterm", "iterm2":
            self = .bundle("com.googlecode.iterm2")
        case "ghostty":
            self = .bundle("com.mitchellh.ghostty")
        case "wezterm":
            self = .bundle("com.github.wez.wezterm")
        case "alacritty":
            self = .bundle("org.alacritty")
        case "kitty":
            self = .bundle("net.kovidgoyal.kitty")
        case "custom":
            let trimmed = customBundleID.trimmingCharacters(in: .whitespacesAndNewlines)
            self = trimmed.isEmpty ? .auto : .bundle(trimmed)
        default:
            if raw.hasPrefix("bundle:") {
                let bundleID = String(raw.dropFirst("bundle:".count)).trimmingCharacters(in: .whitespacesAndNewlines)
                self = bundleID.isEmpty ? .auto : .bundle(bundleID)
            } else {
                self = .auto
            }
        }
    }
}
