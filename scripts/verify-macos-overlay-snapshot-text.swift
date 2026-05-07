#!/usr/bin/env swift
import Foundation
import ImageIO
import Vision

let root = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
let snapshotDir = CommandLine.arguments.dropFirst().first.map { URL(fileURLWithPath: $0, relativeTo: root).standardizedFileURL }
    ?? root.appendingPathComponent(".ai/validation/macos-overlay/headless-snapshots")

let expectations: [String: [String]] = [
    "attention": ["Search Agent Sessions", "Need input", "ATTENTION", "Pull Request", "Open PR"],
    "compose": ["Region: Compose", "Can you summarize", "Send"],
    "diagnostic-empty": ["Foreman control diagnostic", "tmux unavailable", "foreman --doctor"],
    "duplicate-workspace": ["project-alpha startup", "project-alpha review", "notes-vault"],
    "empty": ["No Foreman agents", "agent detector"],
    "error": ["could not load agents", "fake Foreman error"],
    "flash": ["Flash", "Type a label"],
    "long-path": ["Deeply/Nested", "CommandPale"],
    "long-preview": ["overlay preview line", "bounded visual layout"],
    "many": ["agent 19", "workspace-19"],
    "mixed-status": ["Fixture agent", "ATTENTION"],
    "pr-active": ["Pull Request", "Open PR", "Region: PR"],
    "help": ["Foreman Help", "Double-click", "Regions"],
    "help-scrolled": ["Status Legend", "Parity Scope", "Deferred TUI features"],
    "theme-indigo": ["Theme: Indigo", "Pull Request"],
    "settings-general": ["Foreman Settings", "General", "Popup Size", "Default"],
    "theme-terminal": ["Theme: Terminal", "Pull Request"],
]

func recognizedText(from url: URL) throws -> String {
    guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil) else {
        throw NSError(domain: "SnapshotText", code: 1, userInfo: [NSLocalizedDescriptionKey: "could not load image: \(url.path)"])
    }
    let request = VNRecognizeTextRequest()
    request.recognitionLevel = .accurate
    request.usesLanguageCorrection = true
    let handler = VNImageRequestHandler(cgImage: image)
    try handler.perform([request])
    return (request.results ?? [])
        .compactMap { $0.topCandidates(1).first?.string }
        .joined(separator: "\n")
}

var failures: [String] = []
let textDir = snapshotDir.appendingPathComponent("recognized-text")
try FileManager.default.createDirectory(at: textDir, withIntermediateDirectories: true)

for (state, expectedStrings) in expectations.sorted(by: { $0.key < $1.key }) {
    let image = snapshotDir.appendingPathComponent("\(state).png")
    guard FileManager.default.fileExists(atPath: image.path) else {
        failures.append("missing snapshot: \(image.path)")
        continue
    }
    do {
        let text = try recognizedText(from: image)
        try text.write(to: textDir.appendingPathComponent("\(state).txt"), atomically: true, encoding: .utf8)
        let normalized = text.lowercased()
        for expected in expectedStrings {
            if !normalized.contains(expected.lowercased()) {
                failures.append("\(state).png missing OCR text: \(expected)")
            }
        }
    } catch {
        failures.append("\(state).png OCR failed: \(error.localizedDescription)")
    }
}

if !failures.isEmpty {
    fputs(failures.joined(separator: "\n") + "\n", stderr)
    exit(1)
}

print("snapshot OCR semantic assertions passed for \(expectations.count) states")
