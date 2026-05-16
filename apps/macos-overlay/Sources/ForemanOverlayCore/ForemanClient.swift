import Foundation

public struct ForemanClient: Sendable {
    public var foremanPath: String
    public var runner: ProcessRunning
    public var includeAllPanes: Bool
    public var includePullRequests: Bool
    public var includeExtensions: Bool

    public init(foremanPath: String, runner: ProcessRunning = ProcessRunner(), includeAllPanes: Bool = false, includePullRequests: Bool = true, includeExtensions: Bool = true) {
        self.foremanPath = foremanPath
        self.runner = runner
        self.includeAllPanes = includeAllPanes
        self.includePullRequests = includePullRequests
        self.includeExtensions = includeExtensions
    }

    public func agents() async throws -> AgentsResponse {
        do {
            return try await agents(includePullRequests: includePullRequests, includeExtensions: includeExtensions)
        } catch ProcessRunnerError.timedOut where includePullRequests && includeExtensions {
            do {
                return try await agents(includePullRequests: true, includeExtensions: false)
            } catch ProcessRunnerError.timedOut {
                return try await agents(includePullRequests: false, includeExtensions: true)
            }
        } catch ProcessRunnerError.timedOut where includePullRequests {
            return try await agents(includePullRequests: false, includeExtensions: includeExtensions)
        }
    }

    public func initialAgents() async throws -> AgentsResponse {
        do {
            return try await agents(includePullRequests: includePullRequests, includeExtensions: false)
        } catch ProcessRunnerError.timedOut where includePullRequests {
            return try await agents(includePullRequests: false, includeExtensions: false)
        }
    }

    public func agents(includePullRequests: Bool, includeExtensions: Bool) async throws -> AgentsResponse {
        var arguments = ["agents", "--json"]
        if includePullRequests {
            arguments.append("--pull-requests")
        }
        if includeAllPanes {
            arguments.append("--all-panes")
        }
        if includeExtensions {
            arguments.append("--extensions")
        }
        let result = try await runner.run(foremanPath, arguments, stdin: nil)
        guard result.status == 0 else {
            throw OverlayError.commandFailed(result.stderr.isEmpty ? result.stdout : result.stderr)
        }
        return try JSONDecoder().decode(AgentsResponse.self, from: Data(result.stdout.utf8))
    }

    public func extensionCards(forPane paneId: String) async throws -> ExtensionCardsResponse {
        let result = try await runner.run(foremanPath, ["extensions", "--pane", paneId, "--json"], stdin: nil)
        guard result.status == 0 else {
            throw OverlayError.commandFailed(result.stderr.isEmpty ? result.stdout : result.stderr)
        }
        return try JSONDecoder().decode(ExtensionCardsResponse.self, from: Data(result.stdout.utf8))
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
