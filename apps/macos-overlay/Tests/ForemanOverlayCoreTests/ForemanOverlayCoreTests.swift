import XCTest
@testable import ForemanOverlayCore

actor SequencedProcessResults {
    private var results: [(delay: Duration, outcome: Result<ProcessResult, Error>)]

    init(_ results: [(delay: Duration, result: ProcessResult)]) {
        self.results = results.map { ($0.delay, .success($0.result)) }
    }

    init(_ outcomes: [(delay: Duration, outcome: Result<ProcessResult, Error>)]) {
        self.results = outcomes
    }

    func next() async throws -> ProcessResult {
        let item = results.removeFirst()
        try await Task.sleep(for: item.delay)
        return try item.outcome.get()
    }
}

struct SequencedRunner: ProcessRunning {
    let sequence: SequencedProcessResults

    func run(_ executable: String, _ arguments: [String], stdin: String?) async throws -> ProcessResult {
        try await sequence.next()
    }
}

actor RecordedProcessCalls {
    private(set) var arguments: [[String]] = []
    let result: ProcessResult

    init(result: ProcessResult) {
        self.result = result
    }

    func record(_ arguments: [String]) -> ProcessResult {
        self.arguments.append(arguments)
        return result
    }

    func lastArguments() -> [String]? {
        arguments.last
    }
}

struct RecordingRunner: ProcessRunning {
    let calls: RecordedProcessCalls

    func run(_ executable: String, _ arguments: [String], stdin: String?) async throws -> ProcessResult {
        await calls.record(arguments)
    }
}

struct FakeRunner: ProcessRunning {
    var result: ProcessResult
    var expectedStdin: String?
    var expectedArguments: [String]?

    func run(_ executable: String, _ arguments: [String], stdin: String?) async throws -> ProcessResult {
        if let expectedStdin {
            XCTAssertEqual(stdin, expectedStdin)
        }
        if let expectedArguments {
            XCTAssertEqual(arguments, expectedArguments)
        }
        return result
    }
}

@MainActor
final class FakeAppRouter: OverlayAppRouting {
    var didFocusPane = false
    var didOpenSettings = false
    var openedURL: String?

    func overlayDidFocusPane() { didFocusPane = true }
    func overlayOpenSettings() { didOpenSettings = true }
    func overlayOpenURL(_ urlString: String) { openedURL = urlString }
}

final class ForemanOverlayCoreTests: XCTestCase {
    func testDecodesAgentFixture() throws {
        let response = try loadFixture()

        XCTAssertEqual(response.schemaVersion, 1)
        XCTAssertEqual(response.entries.count, 3)
        XCTAssertEqual(response.entries.first?.status, "needs-attention")
        XCTAssertEqual(response.entries.first?.paneId, "%101")
    }

    func testTerminalActivationPreferenceParsing() {
        XCTAssertEqual(OverlayTerminalActivationPreference(raw: "auto"), .auto)
        XCTAssertEqual(OverlayTerminalActivationPreference(raw: "none"), .none)
        XCTAssertEqual(OverlayTerminalActivationPreference(raw: "ghostty"), .bundle("com.mitchellh.ghostty"))
        XCTAssertEqual(OverlayTerminalActivationPreference(raw: "iterm2"), .bundle("com.googlecode.iterm2"))
        XCTAssertEqual(OverlayTerminalActivationPreference(raw: "bundle:com.example.Terminal"), .bundle("com.example.Terminal"))
        XCTAssertEqual(OverlayTerminalActivationPreference(raw: "custom", customBundleID: "com.example.Custom"), .bundle("com.example.Custom"))
        XCTAssertEqual(OverlayTerminalActivationPreference(raw: "custom", customBundleID: "  "), .auto)
        XCTAssertEqual(OverlayTerminalActivationPreference(raw: "surprise"), .auto)
    }

    @MainActor
    func testPreferencesClampAndPersistPopupSize() throws {
        let suite = "foreman-overlay-tests-\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suite)!
        defer { defaults.removePersistentDomain(forName: suite) }
        let preferences = OverlayPreferences(defaults: defaults)

        preferences.popupWidth = 100
        preferences.popupHeight = 2000
        preferences.sortMode = .attentionFirst

        let loaded = OverlayPreferences(defaults: defaults)
        XCTAssertEqual(loaded.popupWidth, OverlayPreferences.minPopupWidth)
        XCTAssertEqual(loaded.popupHeight, OverlayPreferences.maxPopupHeight)
        XCTAssertEqual(loaded.sortMode, .attentionFirst)
    }

    @MainActor
    func testRowPresenterDisambiguatesDuplicateWorkspaces() throws {
        let response = try JSONDecoder().decode(AgentsResponse.self, from: try Data(contentsOf: URL(fileURLWithPath: "Fixtures/agents-duplicate-workspace.json")))
        let first = response.entries[0]
        let second = response.entries[1]

        XCTAssertEqual(OverlayRowPresenter.presentation(for: first, in: response.entries, mode: .smart).title, "project-alpha startup")
        XCTAssertEqual(OverlayRowPresenter.presentation(for: second, in: response.entries, mode: .smart).title, "project-alpha review")
        XCTAssertEqual(OverlayRowPresenter.presentation(for: first, in: response.entries, mode: .workspace).title, "notes-vault")
        XCTAssertEqual(OverlayRowPresenter.presentation(for: first, in: response.entries, mode: .session).title, "notes")
        XCTAssertTrue(OverlayRowPresenter.presentation(for: first, in: response.entries, mode: .smart).subtitle.contains("%35"))
    }

    @MainActor
    func testStorePresentedEntriesUseBatchedRowPresentation() throws {
        let response = try JSONDecoder().decode(AgentsResponse.self, from: try Data(contentsOf: URL(fileURLWithPath: "Fixtures/agents-duplicate-workspace.json")))
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"), preferences: testPreferences())
        store.response = response

        let rows = store.presentedEntries

        XCTAssertEqual(rows.map(\.id), store.entries.map(\.id))
        XCTAssertEqual(rows[0].presentation.title, "project-alpha startup")
        XCTAssertEqual(rows[1].presentation.title, "project-alpha review")
        XCTAssertTrue(rows[0].presentation.subtitle.contains("%35"))
    }

    @MainActor
    func testStoreAppliesPreferencesFiltersAndSort() throws {
        let response = try loadFixture()
        let preferences = OverlayPreferences(defaults: UserDefaults(suiteName: "foreman-overlay-tests-\(UUID().uuidString)")!)
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"), preferences: preferences)
        store.response = response

        preferences.harnessFilter = .codex
        XCTAssertEqual(store.entries.map(\.paneId), ["%103"])

        preferences.harnessFilter = .all
        preferences.statusFilter = .working
        XCTAssertEqual(store.entries.map(\.paneId), ["%102"])

        preferences.statusFilter = .all
        preferences.sortMode = .attentionFirst
        XCTAssertEqual(store.entries.first?.status, "needs-attention")
    }

    @MainActor
    func testAttentionSortUsesRecentActivityAsTiebreaker() throws {
        let response = try loadFixture(named: "agents-many")
        let preferences = testPreferences()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"), preferences: preferences)
        store.response = response

        preferences.sortMode = .attentionFirst

        XCTAssertEqual(store.entries.prefix(3).map(\.paneId), ["%201", "%205", "%209"])
        XCTAssertEqual(OverlaySortMode.attentionFirst.label, "Attention → Recent")
    }

    @MainActor
    func testStoreFiltersByWorkspaceHarnessAndPreview() throws {
        let response = try loadFixture()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"))
        store.response = response

        store.query = "harness"
        XCTAssertEqual(store.entries.map(\.paneId), ["%102"])

        store.query = "codex"
        XCTAssertEqual(store.entries.map(\.paneId), ["%103"])

        store.query = "choose whether"
        XCTAssertEqual(store.entries.map(\.paneId), ["%101"])
    }

    @MainActor
    func testSelectionNormalizesWhenQueryHidesSelectedRow() throws {
        let response = try loadFixture()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"), preferences: testPreferences())
        store.response = response
        store.selectionId = response.entries[1].id

        store.query = "choose whether"

        XCTAssertEqual(store.entries.map(\.paneId), ["%101"])
        XCTAssertEqual(store.selectionId, response.entries[0].id)
        XCTAssertEqual(store.selectedEntry?.paneId, "%101")
    }

    @MainActor
    func testSelectionNormalizesWhenFilterHidesSelectedRow() async throws {
        let response = try loadFixture()
        let preferences = testPreferences()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"), preferences: preferences)
        store.response = response
        store.selectionId = response.entries[2].id

        preferences.harnessFilter = .pi
        try await Task.sleep(for: .milliseconds(80))

        XCTAssertEqual(store.entries.map(\.paneId), ["%101", "%102"])
        XCTAssertEqual(store.selectionId, response.entries[0].id)
        XCTAssertEqual(store.selectedEntry?.paneId, "%101")
    }

    @MainActor
    func testMoveSelectionClampsToAvailableEntries() throws {
        let response = try loadFixture()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"))
        store.response = response
        store.selectionId = response.entries[0].id
        store.previewScrollOffset = 7

        store.moveSelection(delta: 1)
        XCTAssertEqual(store.selectedEntry?.paneId, "%102")
        XCTAssertEqual(store.previewScrollOffset, 0)

        store.moveSelection(delta: 99)
        XCTAssertEqual(store.selectedEntry?.paneId, "%103")

        store.moveSelection(delta: -99)
        XCTAssertEqual(store.selectedEntry?.paneId, "%101")
    }

    @MainActor
    func testRegionCyclingAndComposeMode() throws {
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"), preferences: testPreferences())

        XCTAssertEqual(store.activeRegion, .list)
        store.cycleRegion()
        XCTAssertEqual(store.activeRegion, .detail)
        XCTAssertFalse(store.isComposing)
        store.cycleRegion()
        XCTAssertEqual(store.activeRegion, .compose)
        XCTAssertTrue(store.isComposing)
        store.cycleRegion(reverse: true)
        XCTAssertEqual(store.activeRegion, .detail)
        XCTAssertFalse(store.isComposing)

        let response = try loadFixture()
        store.response = response
        store.selectionId = response.entries[0].id
        store.activateRegion(.list)
        store.cycleRegion()
        XCTAssertEqual(store.activeRegion, .pullRequest)
        store.cycleRegion()
        XCTAssertEqual(store.activeRegion, .detail)
        store.cycleRegion(reverse: true)
        XCTAssertEqual(store.activeRegion, .pullRequest)
    }

    @MainActor
    func testHelpAndComposeCancelBehavior() {
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"))

        store.toggleHelp()
        store.scrollHelp(delta: 99)
        XCTAssertTrue(store.isHelpVisible)
        XCTAssertEqual(store.helpScrollOffset, 6)
        XCTAssertTrue(store.closeHelpOrCancelCompose())
        XCTAssertFalse(store.isHelpVisible)

        store.activateRegion(.compose)
        XCTAssertTrue(store.isComposing)
        XCTAssertTrue(store.closeHelpOrCancelCompose())
        XCTAssertFalse(store.isComposing)
        XCTAssertEqual(store.activeRegion, .list)
        XCTAssertFalse(store.closeHelpOrCancelCompose())
    }

    @MainActor
    func testPreviewScrollClampsToSelectedPreviewLines() throws {
        let response = try loadFixture()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"))
        store.response = response
        store.selectionId = response.entries[0].id

        store.scrollPreview(delta: 99)
        XCTAssertGreaterThan(store.previewScrollOffset, 0)
        store.scrollPreview(delta: -99)
        XCTAssertEqual(store.previewScrollOffset, 0)
    }

    @MainActor
    func testKeyboardReducerHandlesSearchNavigationAndFocusEffects() throws {
        let response = try loadFixture()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"))
        store.response = response
        store.selectionId = response.entries[0].id

        XCTAssertEqual(store.handleKeyboardCommand(.typed("codex")), .none)
        XCTAssertEqual(store.query, "codex")
        XCTAssertEqual(store.entries.map(\.paneId), ["%103"])
        XCTAssertEqual(store.handleKeyboardCommand(.enter(command: false)), .focusSelected)

        XCTAssertEqual(store.handleKeyboardCommand(.deleteBackward), .none)
        XCTAssertEqual(store.query, "code")
        XCTAssertEqual(store.handleKeyboardCommand(.typed(",")), .none)
        XCTAssertEqual(store.query, "code,")
    }

    @MainActor
    func testKeyboardReducerHandlesFlashJumpLabels() throws {
        let response = try loadFixture()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"))
        store.response = response
        store.selectionId = response.entries[0].id

        XCTAssertEqual(store.handleKeyboardCommand(.beginFlash(focusOnMatch: false)), .none)
        XCTAssertTrue(store.isFlashVisible)
        XCTAssertEqual(store.flashLabel(for: response.entries[0]), "A")
        XCTAssertEqual(store.handleKeyboardCommand(.typed("S")), .none)
        XCTAssertFalse(store.isFlashVisible)
        XCTAssertEqual(store.selectedEntry?.paneId, "%102")

        XCTAssertEqual(store.handleKeyboardCommand(.beginFlash(focusOnMatch: true)), .none)
        XCTAssertEqual(store.handleKeyboardCommand(.typed("D")), .focusSelected)
        XCTAssertEqual(store.selectedEntry?.paneId, "%103")
    }

    @MainActor
    func testKeyboardReducerHandlesHelpDetailAndComposeModes() throws {
        let response = try loadFixture()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"), preferences: testPreferences())
        store.response = response
        store.selectionId = response.entries[0].id

        XCTAssertEqual(store.handleKeyboardCommand(.typed("?")), .none)
        XCTAssertTrue(store.isHelpVisible)
        XCTAssertEqual(store.handleKeyboardCommand(.typed("j")), .none)
        XCTAssertEqual(store.helpScrollOffset, 1)
        XCTAssertEqual(store.handleKeyboardCommand(.escape), .none)
        XCTAssertFalse(store.isHelpVisible)

        XCTAssertEqual(store.handleKeyboardCommand(.tab(reverse: false)), .none)
        XCTAssertEqual(store.activeRegion, .pullRequest)
        XCTAssertEqual(store.handleKeyboardCommand(.enter(command: false)), .openPullRequest)

        XCTAssertEqual(store.handleKeyboardCommand(.tab(reverse: false)), .none)
        XCTAssertEqual(store.activeRegion, .detail)
        XCTAssertEqual(store.handleKeyboardCommand(.typed("j")), .none)
        XCTAssertGreaterThan(store.previewScrollOffset, 0)
        XCTAssertEqual(store.handleKeyboardCommand(.enter(command: false)), .focusSelected)

        XCTAssertEqual(store.handleKeyboardCommand(.tab(reverse: false)), .none)
        XCTAssertEqual(store.activeRegion, .compose)
        XCTAssertTrue(store.isComposing)
        XCTAssertEqual(store.handleKeyboardCommand(.enter(command: false)), .passThrough)
        XCTAssertEqual(store.handleKeyboardCommand(.enter(command: true)), .sendToSelected)
        XCTAssertEqual(store.handleKeyboardCommand(.openSettings), .openSettings)
        XCTAssertEqual(store.handleKeyboardCommand(.escape), .none)
        XCTAssertFalse(store.isComposing)
    }

    @MainActor
    func testPullRequestRegionIsSkippedWhenSelectionHasNoPR() throws {
        let response = try loadFixture()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"), preferences: testPreferences())
        store.response = response
        store.selectionId = response.entries[1].id

        XCTAssertNil(store.selectedEntry?.pullRequest)
        XCTAssertEqual(store.availableFocusRegions, [.list, .detail, .compose])
        XCTAssertEqual(store.handleKeyboardCommand(.tab(reverse: false)), .none)
        XCTAssertEqual(store.activeRegion, .detail)

        store.activateRegion(.pullRequest)
        XCTAssertEqual(store.activeRegion, .detail)
    }

    @MainActor
    func testIncludeAllPanesPreferenceSchedulesReload() async throws {
        let fixture = String(decoding: try fixtureData(), as: UTF8.self)
        let calls = RecordedProcessCalls(result: ProcessResult(stdout: fixture, stderr: "", status: 0))
        let preferences = testPreferences()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/fake/foreman", runner: RecordingRunner(calls: calls)), preferences: preferences)

        preferences.includeAllPanes = true
        try await Task.sleep(for: .milliseconds(250))

        let lastArguments = await calls.lastArguments()
        XCTAssertEqual(lastArguments, ["agents", "--json", "--pull-requests", "--all-panes"])
        XCTAssertFalse(store.isLoading)
    }

    @MainActor
    func testReloadDiscardsStaleResponses() async throws {
        let firstJSON = String(decoding: try fixtureData(), as: UTF8.self)
        let secondJSON = firstJSON.replacingOccurrences(of: "%101", with: "%999")
        let runner = SequencedRunner(sequence: SequencedProcessResults([
            (.milliseconds(200), ProcessResult(stdout: firstJSON, stderr: "", status: 0)),
            (.milliseconds(10), ProcessResult(stdout: secondJSON, stderr: "", status: 0)),
        ]))
        let store = OverlayStore(client: ForemanClient(foremanPath: "/fake/foreman", runner: runner), preferences: testPreferences())

        store.reload()
        try await Task.sleep(for: .milliseconds(20))
        store.reload()
        try await Task.sleep(for: .milliseconds(350))

        XCTAssertFalse(store.isLoading)
        XCTAssertEqual(store.response?.entries.first?.paneId, "%999")
        XCTAssertEqual(store.lastReloadSummary, "Loaded 3 agent session(s)")
    }

    @MainActor
    func testStoreRoutesAppEffectsThroughRouter() throws {
        let response = try loadFixture()
        let store = OverlayStore(client: ForemanClient(foremanPath: "/bin/false"), preferences: testPreferences())
        let router = FakeAppRouter()
        store.response = response
        store.selectionId = response.entries[0].id
        store.appRouter = router

        store.requestOpenSettings()
        XCTAssertTrue(router.didOpenSettings)

        store.openSelectedPullRequest()
        XCTAssertEqual(router.openedURL, "https://github.com/example/foreman/pull/123")
    }

    func testForemanClientDecodesAgentsFromRunnerStdout() async throws {
        let fixture = try fixtureData()
        let runner = FakeRunner(
            result: ProcessResult(stdout: String(decoding: fixture, as: UTF8.self), stderr: "", status: 0),
            expectedArguments: ["agents", "--json", "--pull-requests"]
        )
        let client = ForemanClient(foremanPath: "/fake/foreman", runner: runner)

        let response = try await client.agents()

        XCTAssertEqual(response.entries.first?.paneId, "%101")
    }

    func testForemanClientCanRequestAllPanes() async throws {
        let fixture = try fixtureData()
        let runner = FakeRunner(
            result: ProcessResult(stdout: String(decoding: fixture, as: UTF8.self), stderr: "", status: 0),
            expectedArguments: ["agents", "--json", "--pull-requests", "--all-panes"]
        )
        let client = ForemanClient(foremanPath: "/fake/foreman", runner: runner, includeAllPanes: true)

        _ = try await client.agents()
    }

    func testForemanClientFallsBackWhenPullRequestLookupTimesOut() async throws {
        let fixture = try fixtureData()
        let runner = SequencedRunner(sequence: SequencedProcessResults([
            (.milliseconds(1), .failure(ProcessRunnerError.timedOut(seconds: 10))),
            (.milliseconds(1), .success(ProcessResult(stdout: String(decoding: fixture, as: UTF8.self), stderr: "", status: 0))),
        ]))
        let client = ForemanClient(foremanPath: "/fake/foreman", runner: runner, includeAllPanes: true)

        let response = try await client.agents()

        XCTAssertEqual(response.entries.first?.paneId, "%101")
    }

    func testForemanClientSurfacesCommandFailure() async throws {
        let runner = FakeRunner(result: ProcessResult(stdout: "", stderr: "tmux unavailable", status: 1))
        let client = ForemanClient(foremanPath: "/fake/foreman", runner: runner)

        do {
            _ = try await client.agents()
            XCTFail("expected agents to fail")
        } catch {
            XCTAssertTrue(error.localizedDescription.contains("tmux unavailable"))
        }
    }

    func testForemanClientSurfacesInvalidJSON() async throws {
        let runner = FakeRunner(result: ProcessResult(stdout: "not json", stderr: "", status: 0))
        let client = ForemanClient(foremanPath: "/fake/foreman", runner: runner)

        do {
            _ = try await client.agents()
            XCTFail("expected JSON decode to fail")
        } catch {
            XCTAssertTrue(String(describing: error).contains("dataCorrupted") || String(describing: error).contains("not valid JSON"))
        }
    }

    func testProcessRunnerReportsMissingExecutable() async throws {
        do {
            _ = try await ProcessRunner(timeoutSeconds: 1).run("/definitely/missing/foreman", [], stdin: nil)
            XCTFail("expected missing executable to fail")
        } catch let error as ProcessRunnerError {
            XCTAssertTrue(error.localizedDescription.contains("executable not found"))
        }
    }

    func testProcessRunnerHardTimesOutTermIgnoringProcess() async throws {
        let start = Date()
        do {
            _ = try await ProcessRunner(timeoutSeconds: 0.1).run("/bin/sh", ["-c", "trap '' TERM; sleep 5"], stdin: nil)
            XCTFail("expected timeout")
        } catch let error as ProcessRunnerError {
            XCTAssertTrue(error.localizedDescription.contains("timed out"))
            XCTAssertLessThan(Date().timeIntervalSince(start), 2)
        }
    }

    func testProcessRunnerDrainsLargeStderr() async throws {
        let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let script = tempDir.appendingPathComponent("large-stderr.sh")
        try "#!/bin/sh\nyes e | head -c 300000 >&2\nexit 7\n".write(to: script, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: script.path)

        let result = try await ProcessRunner(timeoutSeconds: 5).run(script.path, [], stdin: nil)

        XCTAssertEqual(result.status, 7)
        XCTAssertEqual(result.stderr.count, 300000)
    }

    func testProcessRunnerDrainsLargeStdout() async throws {
        let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let script = tempDir.appendingPathComponent("large-output.sh")
        try "#!/bin/sh\nyes x | head -c 300000\n".write(to: script, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: script.path)

        let result = try await ProcessRunner(timeoutSeconds: 5).run(script.path, [], stdin: nil)

        XCTAssertEqual(result.status, 0)
        XCTAssertEqual(result.stdout.count, 300000)
    }

    @MainActor
    private func testPreferences() -> OverlayPreferences {
        let suite = "foreman-overlay-tests-\(UUID().uuidString)"
        return OverlayPreferences(defaults: UserDefaults(suiteName: suite)!)
    }

    private func loadFixture() throws -> AgentsResponse {
        try loadFixture(named: "agents-attention")
    }

    private func loadFixture(named name: String) throws -> AgentsResponse {
        try JSONDecoder().decode(AgentsResponse.self, from: fixtureData(named: name))
    }

    private func fixtureData() throws -> Data {
        try fixtureData(named: "agents-attention")
    }

    private func fixtureData(named name: String) throws -> Data {
        let sourceURL = URL(fileURLWithPath: "Fixtures/\(name).json")
        if FileManager.default.fileExists(atPath: sourceURL.path) {
            return try Data(contentsOf: sourceURL)
        }
        let url = Bundle.module.url(forResource: name, withExtension: "json", subdirectory: "Fixtures")!
        return try Data(contentsOf: url)
    }
}
