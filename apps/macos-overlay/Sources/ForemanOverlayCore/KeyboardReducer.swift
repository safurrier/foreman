import Foundation

public enum OverlayKeyboardCommand: Equatable, Sendable {
    case escape
    case beginFlash(focusOnMatch: Bool)
    case tab(reverse: Bool)
    case moveUp
    case moveDown
    case pageUp
    case pageDown
    case enter(command: Bool)
    case refresh
    case cycleTheme
    case openSettings
    case deleteBackward
    case typed(String)
}

@MainActor
public protocol OverlayAppRouting: AnyObject {
    func overlayDidFocusPane()
    func overlayOpenSettings()
    func overlayOpenURL(_ urlString: String)
}

public enum OverlayKeyboardEffect: Equatable, Sendable {
    case none
    case passThrough
    case hideOverlay
    case focusSelected
    case sendToSelected
    case openPullRequest
    case reload
    case openSettings
}

