import AppKit
import Foundation
import KeyboardShortcuts
import SwiftUI

import ForemanOverlayCore
import ForemanOverlayUI

var root = ProcessInfo.processInfo.environment["FOREMAN_REPO_ROOT"]
    .map { URL(fileURLWithPath: $0).standardizedFileURL }
    ?? URL(fileURLWithPath: FileManager.default.currentDirectoryPath).standardizedFileURL
let arguments = CommandLine.arguments.dropFirst()

var outputDir = root.appendingPathComponent(".ai/validation/macos-overlay/snapshots")
var outputDirWasSet = false
var requestedState = "all"
var listStates = false
var width: CGFloat = 820
var height: CGFloat = 560

var iterator = arguments.makeIterator()
while let arg = iterator.next() {
    switch arg {
    case "--repo-root":
        guard let value = iterator.next() else { fatalError("--repo-root requires a value") }
        root = URL(fileURLWithPath: value).standardizedFileURL
        if !outputDirWasSet {
            outputDir = root.appendingPathComponent(".ai/validation/macos-overlay/snapshots")
        }
    case "--output-dir":
        guard let value = iterator.next() else { fatalError("--output-dir requires a value") }
        outputDirWasSet = true
        outputDir = URL(fileURLWithPath: value, relativeTo: root).standardizedFileURL
    case "--state":
        guard let value = iterator.next() else { fatalError("--state requires a value") }
        requestedState = value
    case "--list-states":
        listStates = true
    case "--width":
        guard let value = iterator.next(), let parsed = Double(value) else { fatalError("--width requires a number") }
        width = CGFloat(parsed)
    case "--height":
        guard let value = iterator.next(), let parsed = Double(value) else { fatalError("--height requires a number") }
        height = CGFloat(parsed)
    default:
        fatalError("unknown argument: \(arg)")
    }
}

struct SnapshotState {
    let name: String
    let fixture: String?
    let errorMessage: String?
    let configure: @MainActor (OverlayStore) -> Void
    let renderSettings: Bool

    init(name: String, fixture: String? = nil, errorMessage: String? = nil, renderSettings: Bool = false, configure: @escaping @MainActor (OverlayStore) -> Void = { _ in }) {
        self.name = name
        self.fixture = fixture
        self.errorMessage = errorMessage
        self.configure = configure
        self.renderSettings = renderSettings
    }
}

let fixturesDir = root.appendingPathComponent("apps/macos-overlay/Fixtures")
let states: [SnapshotState] = [
    SnapshotState(name: "attention", fixture: "agents-attention.json"),
    SnapshotState(name: "empty", fixture: "agents-empty.json"),
    SnapshotState(name: "diagnostic-empty", fixture: "agents-empty-diagnostics.json"),
    SnapshotState(name: "error", errorMessage: "fixture tmux unavailable: fake Foreman error"),
    SnapshotState(name: "long-path", fixture: "agents-long-path.json"),
    SnapshotState(name: "long-preview", fixture: "agents-long-preview.json", configure: { store in
        store.activateRegion(.detail)
        store.scrollPreview(delta: 18)
    }),
    SnapshotState(name: "many", fixture: "agents-many.json", configure: { store in
        store.moveSelection(delta: 18)
    }),
    SnapshotState(name: "mixed-status", fixture: "agents-mixed-status.json"),
    SnapshotState(name: "pr-active", fixture: "agents-attention.json", configure: { store in
        store.activateRegion(.pullRequest)
    }),
    SnapshotState(name: "duplicate-workspace", fixture: "agents-duplicate-workspace.json"),
    SnapshotState(name: "compose", fixture: "agents-attention.json", configure: { store in
        store.activateRegion(.compose)
        store.composeText = "Can you summarize the current validation failure?"
    }),
    SnapshotState(name: "flash", fixture: "agents-attention.json", configure: { store in
        store.beginFlash()
    }),
    SnapshotState(name: "help", fixture: "agents-attention.json", configure: { store in
        store.toggleHelp()
    }),
    SnapshotState(name: "help-scrolled", fixture: "agents-attention.json", configure: { store in
        store.toggleHelp()
        store.scrollHelp(delta: 6)
    }),
    SnapshotState(name: "theme-indigo", fixture: "agents-attention.json", configure: { store in
        store.theme = .indigo
    }),
    SnapshotState(name: "theme-terminal", fixture: "agents-attention.json", configure: { store in
        store.theme = .terminal
    }),
    SnapshotState(name: "settings-general", renderSettings: true),
]

if listStates {
    print(states.map(\.name).joined(separator: "\n"))
    exit(0)
}

let selectedStates: [SnapshotState]
if requestedState == "all" {
    selectedStates = states
} else if let state = states.first(where: { $0.name == requestedState }) {
    selectedStates = [state]
} else {
    fatalError("unknown state: \(requestedState)")
}

try FileManager.default.createDirectory(at: outputDir, withIntermediateDirectories: true)
_ = NSApplication.shared

@MainActor
func makeStore(for state: SnapshotState) throws -> OverlayStore {
    let store = OverlayStore(client: ForemanClient(foremanPath: "/usr/bin/false"))
    if let fixture = state.fixture {
        let data = try Data(contentsOf: fixturesDir.appendingPathComponent(fixture))
        store.response = try JSONDecoder().decode(AgentsResponse.self, from: data)
        store.selectionId = store.response?.entries.first?.id
    }
    if let errorMessage = state.errorMessage {
        store.errorMessage = errorMessage
    }
    state.configure(store)
    return store
}

@MainActor
func render(state: SnapshotState) throws -> URL {
    let store = try makeStore(for: state)
    let view: AnyView
    let renderWidth: CGFloat
    let renderHeight: CGFloat
    if state.renderSettings {
        let shortcutDefaultsKey = "KeyboardShortcuts_toggleForemanOverlay"
        let previousShortcut = UserDefaults.standard.object(forKey: shortcutDefaultsKey)
        KeyboardShortcuts.reset(.toggleForemanOverlay)
        defer {
            if let previousShortcut {
                UserDefaults.standard.set(previousShortcut, forKey: shortcutDefaultsKey)
            } else {
                UserDefaults.standard.removeObject(forKey: shortcutDefaultsKey)
            }
        }
        view = AnyView(SettingsView(preferences: store.preferences, onClearShortcut: {}, onRestoreDefault: {}))
        renderWidth = 640
        renderHeight = 560
    } else {
        view = AnyView(OverlayView(store: store, autoReload: false))
        renderWidth = width
        renderHeight = height
    }
    let hostingView = NSHostingView(rootView: view)
    hostingView.frame = NSRect(x: 0, y: 0, width: renderWidth, height: renderHeight)
    hostingView.wantsLayer = true
    let window = NSWindow(contentRect: hostingView.frame, styleMask: [.borderless], backing: .buffered, defer: false)
    window.contentView = hostingView
    window.orderOut(nil)
    hostingView.layoutSubtreeIfNeeded()
    RunLoop.main.run(until: Date().addingTimeInterval(0.2))
    hostingView.layoutSubtreeIfNeeded()

    guard let representation = hostingView.bitmapImageRepForCachingDisplay(in: hostingView.bounds) else {
        throw NSError(domain: "ForemanOverlaySnapshot", code: 1, userInfo: [NSLocalizedDescriptionKey: "could not create bitmap representation"])
    }
    representation.size = hostingView.bounds.size
    hostingView.cacheDisplay(in: hostingView.bounds, to: representation)
    guard let png = representation.representation(using: .png, properties: [:]) else {
        throw NSError(domain: "ForemanOverlaySnapshot", code: 2, userInfo: [NSLocalizedDescriptionKey: "could not encode PNG"])
    }
    let output = outputDir.appendingPathComponent("\(state.name).png")
    try png.write(to: output)
    return output
}

@MainActor
func displayPath(_ url: URL) -> String {
    let rootPath = root.standardizedFileURL.path
    let path = url.standardizedFileURL.path
    if path.hasPrefix(rootPath + "/") {
        return String(path.dropFirst(rootPath.count + 1))
    }
    return path
}

let started = ISO8601DateFormatter().string(from: Date())
var lines = ["# macOS Overlay Headless Snapshots", "", "- Started: \(started)", "- Output: \(displayPath(outputDir))", "", "## Files"]

for state in selectedStates {
    let output = try await MainActor.run { try render(state: state) }
    lines.append("- \(state.name): \(displayPath(output))")
    print("rendered \(state.name): \(output.path)")
}

lines.append("- Finished: \(ISO8601DateFormatter().string(from: Date()))")
try lines.joined(separator: "\n").write(to: outputDir.appendingPathComponent("summary.md"), atomically: true, encoding: .utf8)
