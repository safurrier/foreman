import Foundation

public struct ForemanClient: Sendable {
    public var foremanPath: String
    public var runner: ProcessRunning
    public var includeAllPanes: Bool
    public var includePullRequests: Bool

    public init(foremanPath: String, runner: ProcessRunning = ProcessRunner(), includeAllPanes: Bool = false, includePullRequests: Bool = true) {
        self.foremanPath = foremanPath
        self.runner = runner
        self.includeAllPanes = includeAllPanes
        self.includePullRequests = includePullRequests
    }

    public func agents() async throws -> AgentsResponse {
        do {
            return try await agents(includePullRequests: includePullRequests)
        } catch ProcessRunnerError.timedOut where includePullRequests {
            return try await agents(includePullRequests: false)
        }
    }

    private func agents(includePullRequests: Bool) async throws -> AgentsResponse {
        var arguments = ["agents", "--json"]
        if includePullRequests {
            arguments.append("--pull-requests")
        }
        if includeAllPanes {
            arguments.append("--all-panes")
        }
        let result = try await runner.run(foremanPath, arguments, stdin: nil)
        guard result.status == 0 else {
            throw OverlayError.commandFailed(result.stderr.isEmpty ? result.stdout : result.stderr)
        }
        return try JSONDecoder().decode(AgentsResponse.self, from: Data(result.stdout.utf8))
    }

    public func focus(paneId: String) async throws {
        let result = try await runner.run(foremanPath, ["focus", "--pane", paneId, "--json"], stdin: nil)
        guard result.status == 0 else {
            throw OverlayError.commandFailed(result.stderr.isEmpty ? result.stdout : result.stderr)
        }
    }

    public func send(paneId: String, text: String) async throws {
        let result = try await runner.run(foremanPath, ["send", "--pane", paneId, "--stdin", "--json"], stdin: text)
        guard result.status == 0 else {
            throw OverlayError.commandFailed(result.stderr.isEmpty ? result.stdout : result.stderr)
        }
    }
}

public enum OverlayError: Error, LocalizedError, Sendable {
    case commandFailed(String)

    public var errorDescription: String? {
        switch self {
        case .commandFailed(let message):
            message.trimmingCharacters(in: .whitespacesAndNewlines)
        }
    }
}
