import Darwin
import Foundation

public struct ProcessResult: Sendable {
    public let stdout: String
    public let stderr: String
    public let status: Int32

    public init(stdout: String, stderr: String, status: Int32) {
        self.stdout = stdout
        self.stderr = stderr
        self.status = status
    }
}

public protocol ProcessRunning: Sendable {
    func run(_ executable: String, _ arguments: [String], stdin: String?) async throws -> ProcessResult
}

public enum ProcessRunnerError: Error, LocalizedError, Sendable {
    case executableNotFound(String)
    case launchFailed(String)
    case timedOut(seconds: TimeInterval)

    public var errorDescription: String? {
        switch self {
        case .executableNotFound(let executable):
            "executable not found: \(executable)"
        case .launchFailed(let message):
            "process failed to launch: \(message)"
        case .timedOut(let seconds):
            "process timed out after \(seconds) second(s)"
        }
    }
}

private final class ProcessCancellationHandle: @unchecked Sendable {
    private let lock = NSLock()
    private var process: Process?
    private var cancelled = false

    func setProcess(_ process: Process) throws {
        lock.lock()
        if cancelled {
            lock.unlock()
            throw CancellationError()
        }
        self.process = process
        lock.unlock()
    }

    func cancel() {
        lock.lock()
        cancelled = true
        let process = self.process
        lock.unlock()
        guard let process, process.isRunning else { return }
        process.terminate()
        DispatchQueue.global(qos: .utility).asyncAfter(deadline: .now() + 0.5) {
            if process.isRunning {
                kill(process.processIdentifier, SIGKILL)
            }
        }
    }

    func clear() {
        lock.lock()
        process = nil
        lock.unlock()
    }

    var isCancelled: Bool {
        lock.lock()
        let value = cancelled
        lock.unlock()
        return value
    }
}

private final class DataBox: @unchecked Sendable {
    private let lock = NSLock()
    private var value = Data()

    func append(_ data: Data) {
        lock.lock()
        value.append(data)
        lock.unlock()
    }

    func get() -> Data {
        lock.lock()
        let data = value
        lock.unlock()
        return data
    }
}

public struct ProcessRunner: ProcessRunning {
    public var timeoutSeconds: TimeInterval

    public init(timeoutSeconds: TimeInterval = 10) {
        self.timeoutSeconds = timeoutSeconds
    }

    public func run(_ executable: String, _ arguments: [String], stdin: String? = nil) async throws -> ProcessResult {
        let cancellation = ProcessCancellationHandle()
        let task = Task.detached(priority: .userInitiated) {
            try runProcessBlocking(executable, arguments, stdin: stdin, timeoutSeconds: timeoutSeconds, cancellation: cancellation)
        }
        return try await withTaskCancellationHandler {
            try await task.value
        } onCancel: {
            cancellation.cancel()
            task.cancel()
        }
    }
}

private func runProcessBlocking(
    _ executable: String,
    _ arguments: [String],
    stdin: String?,
    timeoutSeconds: TimeInterval,
    cancellation: ProcessCancellationHandle
) throws -> ProcessResult {
    let resolved = try resolveExecutable(executable, arguments: arguments)
    let process = Process()
    process.executableURL = URL(fileURLWithPath: resolved.executable)
    process.arguments = resolved.arguments
    process.environment = cliEnvironment()

    let stdout = Pipe()
    let stderr = Pipe()
    process.standardOutput = stdout
    process.standardError = stderr

    let stdinPipe: Pipe?
    if stdin != nil {
        let input = Pipe()
        process.standardInput = input
        stdinPipe = input
    } else {
        stdinPipe = nil
    }

    let stdoutBox = DataBox()
    let stderrBox = DataBox()

    stdout.fileHandleForReading.readabilityHandler = { handle in
        let data = handle.availableData
        if !data.isEmpty { stdoutBox.append(data) }
    }
    stderr.fileHandleForReading.readabilityHandler = { handle in
        let data = handle.availableData
        if !data.isEmpty { stderrBox.append(data) }
    }

    do {
        try cancellation.setProcess(process)
        try process.run()
        if cancellation.isCancelled {
            cancellation.cancel()
            cleanupPipes(stdout: stdout, stderr: stderr, stdin: stdinPipe)
            cancellation.clear()
            throw CancellationError()
        }
    } catch is CancellationError {
        cleanupPipes(stdout: stdout, stderr: stderr, stdin: stdinPipe)
        cancellation.clear()
        throw CancellationError()
    } catch {
        cleanupPipes(stdout: stdout, stderr: stderr, stdin: stdinPipe)
        cancellation.clear()
        throw ProcessRunnerError.launchFailed(error.localizedDescription)
    }

    if let stdin, let stdinPipe {
        stdinPipe.fileHandleForWriting.write(Data(stdin.utf8))
        try? stdinPipe.fileHandleForWriting.close()
    }

    let deadline = Date().addingTimeInterval(timeoutSeconds)
    while process.isRunning && Date() < deadline {
        if Task.isCancelled || cancellation.isCancelled {
            cancellation.cancel()
            cleanupPipes(stdout: stdout, stderr: stderr, stdin: stdinPipe)
            cancellation.clear()
            throw CancellationError()
        }
        Thread.sleep(forTimeInterval: 0.02)
    }

    if process.isRunning {
        process.terminate()
        let terminationDeadline = Date().addingTimeInterval(0.5)
        while process.isRunning && Date() < terminationDeadline {
            Thread.sleep(forTimeInterval: 0.02)
        }
        if process.isRunning {
            kill(process.processIdentifier, SIGKILL)
        }
        cleanupPipes(stdout: stdout, stderr: stderr, stdin: stdinPipe)
        cancellation.clear()
        throw ProcessRunnerError.timedOut(seconds: timeoutSeconds)
    }

    process.waitUntilExit()
    cleanupPipes(stdout: stdout, stderr: stderr, stdin: stdinPipe)
    cancellation.clear()

    return ProcessResult(
        stdout: String(data: stdoutBox.get(), encoding: .utf8) ?? "",
        stderr: String(data: stderrBox.get(), encoding: .utf8) ?? "",
        status: process.terminationStatus
    )
}

private func cliEnvironment() -> [String: String] {
    var environment = ProcessInfo.processInfo.environment
    let standardPath = [
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        "/usr/local/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
    ].joined(separator: ":")
    if let path = environment["PATH"], !path.isEmpty {
        environment["PATH"] = path + ":" + standardPath
    } else {
        environment["PATH"] = standardPath
    }
    return environment
}

private func cleanupPipes(stdout: Pipe, stderr: Pipe, stdin: Pipe?) {
    stdout.fileHandleForReading.readabilityHandler = nil
    stderr.fileHandleForReading.readabilityHandler = nil
    try? stdout.fileHandleForReading.close()
    try? stderr.fileHandleForReading.close()
    try? stdin?.fileHandleForWriting.close()
}

private func resolveExecutable(_ executable: String, arguments: [String]) throws -> (executable: String, arguments: [String]) {
    if executable.contains("/") {
        guard FileManager.default.isExecutableFile(atPath: executable) else {
            throw ProcessRunnerError.executableNotFound(executable)
        }
        return (executable, arguments)
    }
    return ("/usr/bin/env", [executable] + arguments)
}
