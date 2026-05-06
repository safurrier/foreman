import XCTest

final class ForemanOverlayUITests: XCTestCase {
    func testFakeForemanEventGauntlet() throws {
        guard ProcessInfo.processInfo.environment["FOREMAN_OVERLAY_RUN_UI_TESTS"] == "1" else {
            throw XCTSkip("Set FOREMAN_OVERLAY_RUN_UI_TESTS=1 to run local macOS UI event gauntlet")
        }

        let packageDir = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        let repoRoot = packageDir
            .deletingLastPathComponent()
            .deletingLastPathComponent()

        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = ["bash", repoRoot.appendingPathComponent("scripts/macos-overlay-gauntlet.sh").path]
        process.currentDirectoryURL = repoRoot
        var environment = ProcessInfo.processInfo.environment
        environment["FOREMAN_OVERLAY_SKIP_BUILD"] = "1"
        process.environment = environment

        let output = Pipe()
        process.standardOutput = output
        process.standardError = output
        try process.run()
        process.waitUntilExit()

        let data = output.fileHandleForReading.readDataToEndOfFile()
        let text = String(decoding: data, as: UTF8.self)
        XCTAssertEqual(process.terminationStatus, 0, text)
    }
}
