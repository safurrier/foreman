#!/usr/bin/env python3
"""Live Foreman source-companion smoke harness.

This is intentionally standard-library only so agents can run it on Mac and Coder
without bootstrapping Python dependencies. It orchestrates real Foreman binaries,
SSH/SCP where requested, and writes structured artifacts for HK evidence.
"""

from __future__ import annotations

import argparse
import json
import os
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REPO = Path(__file__).resolve().parents[1]
DEFAULT_CODER_HOST = "coder.alex-furrier-dev-gpu-1"


@dataclass
class CmdResult:
    args: list[str]
    returncode: int
    stdout: str
    stderr: str


def run(args: list[str], *, input_text: str | None = None, timeout: int = 120, cwd: Path | None = None) -> CmdResult:
    proc = subprocess.run(
        args,
        input=input_text,
        cwd=str(cwd) if cwd else None,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )
    result = CmdResult(args=args, returncode=proc.returncode, stdout=proc.stdout, stderr=proc.stderr)
    if proc.returncode != 0:
        raise RuntimeError(
            f"command failed ({proc.returncode}): {' '.join(args)}\nSTDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
        )
    return result


def json_run(args: list[str], **kwargs: Any) -> dict[str, Any]:
    result = run(args, **kwargs)
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"expected JSON from {' '.join(args)}: {exc}\n{result.stdout}") from exc


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def build_candidate(binary: Path) -> None:
    run(["cargo", "build", "--locked"], cwd=REPO, timeout=600)
    built = REPO / "target" / "debug" / "foreman"
    if binary != built:
        binary.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(built, binary)


def base_config(snapshot: Path | None = None, companion_endpoint: str | None = None) -> str:
    chunks = ["[sources]\ndefault_scope = \"all\"\nquery_timeout_ms = 1000\n"]
    if snapshot:
        chunks.append(
            f"""
[sources.mac]
kind = "snapshot"
label = "Mac"
path = "{snapshot}"
"""
        )
    if companion_endpoint:
        chunks.append(
            f"""
[sources.mac-live]
kind = "companion"
label = "Mac Live"
endpoint = "{companion_endpoint}"
timeout_ms = 1000
allow_send = true
"""
        )
    return "\n".join(chunks)


def assert_source_counts(payload: dict[str, Any], required_sources: set[str]) -> dict[str, int]:
    entries = payload.get("entries", [])
    counts: dict[str, int] = {}
    for entry in entries:
        source = entry.get("sourceId", "")
        counts[source] = counts.get(source, 0) + 1
    missing = sorted(source for source in required_sources if counts.get(source, 0) <= 0)
    if missing:
        raise AssertionError(f"missing expected sources {missing}; counts={counts}; payload keys={payload.keys()}")
    return counts


def snapshot(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    binary = Path(args.binary)
    if args.build:
        build_candidate(binary)
    snapshot_path = artifact_dir / "mac-source.agents.json"
    result = json_run(
        [str(binary), "sources", "snapshot", "--source-id", "mac", "--output", str(snapshot_path), "--all-panes", "--json"],
        timeout=120,
    )
    write(artifact_dir / "snapshot-result.json", json.dumps(result, indent=2) + "\n")
    return {"snapshot": result, "snapshotPath": str(snapshot_path)}


def companion_local(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    binary = Path(args.binary)
    if args.build:
        build_candidate(binary)
    ready_file = artifact_dir / "companion-ready.txt"
    config_file = artifact_dir / "companion-client.toml"
    server = subprocess.Popen(
        [
            str(binary),
            "companion",
            "serve",
            "--bind",
            "127.0.0.1:0",
            "--allow-send",
            "--max-requests",
            "1",
            "--ready-file",
            str(ready_file),
            "--json",
        ],
        cwd=str(REPO),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        deadline = time.time() + 10
        while time.time() < deadline and not ready_file.exists():
            if server.poll() is not None:
                out, err = server.communicate(timeout=1)
                raise RuntimeError(f"companion server exited early\nSTDOUT:\n{out}\nSTDERR:\n{err}")
            time.sleep(0.05)
        if not ready_file.exists():
            raise TimeoutError("companion server did not write ready file")
        endpoint = ready_file.read_text().strip()
        write(config_file, base_config(companion_endpoint=endpoint))
        payload = json_run(
            [str(binary), "--config-file", str(config_file), "agents", "--json", "--all-panes"],
            timeout=120,
        )
        counts = assert_source_counts(payload, {"local", "mac-live"})
        out, err = server.communicate(timeout=10)
        write(artifact_dir / "companion-server.stdout.log", out)
        write(artifact_dir / "companion-server.stderr.log", err)
        write(artifact_dir / "companion-agents-response.json", json.dumps(payload, indent=2) + "\n")
        return {"endpoint": endpoint, "counts": counts, "entryCount": len(payload.get("entries", []))}
    finally:
        if server.poll() is None:
            server.terminate()
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()


def install_coder_candidate(args: argparse.Namespace) -> None:
    remote_dir = args.coder_workdir.rstrip("/") + "/repo"
    quoted_remote_dir = shlex.quote(remote_dir)
    run(["ssh", args.coder_host, f"rm -rf {quoted_remote_dir} && mkdir -p {quoted_remote_dir}"], timeout=120)
    run(
        [
            "rsync",
            "-a",
            "--delete",
            "--exclude",
            "target/",
            "--exclude",
            ".git/",
            "--exclude",
            ".pi/",
            str(REPO) + "/",
            f"{args.coder_host}:{remote_dir}/",
        ],
        timeout=300,
    )
    run(
        [
            "ssh",
            args.coder_host,
            f"cd {quoted_remote_dir} && cargo install --path . --locked --force",
        ],
        timeout=900,
    )


def coder_snapshot(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    binary = Path(args.binary)
    if args.build:
        build_candidate(binary)
    if args.install_coder:
        install_coder_candidate(args)
    snap = snapshot(args, artifact_dir)
    snapshot_path = Path(snap["snapshotPath"])
    remote_dir = args.coder_workdir.rstrip("/")
    remote_snapshot = f"{remote_dir}/mac-source.agents.json"
    remote_config = f"{remote_dir}/coder-snapshot.toml"
    run(["ssh", args.coder_host, f"mkdir -p {shlex.quote(remote_dir)}"], timeout=60)
    run(["scp", str(snapshot_path), f"{args.coder_host}:{remote_snapshot}"], timeout=120)
    config_text = base_config(snapshot=Path(remote_snapshot))
    local_config = artifact_dir / "coder-snapshot.toml"
    write(local_config, config_text)
    run(["scp", str(local_config), f"{args.coder_host}:{remote_config}"], timeout=120)
    payload_text = run(
        [
            "ssh",
            args.coder_host,
            f"{shlex.quote(args.coder_foreman)} --config-file {shlex.quote(remote_config)} agents --json --all-panes",
        ],
        timeout=120,
    ).stdout
    payload = json.loads(payload_text)
    counts = assert_source_counts(payload, {"local", "mac"})
    write(artifact_dir / "coder-agents-response.json", json.dumps(payload, indent=2) + "\n")
    return {"entryCount": len(payload.get("entries", [])), "bySource": counts, "diagnostics": payload.get("sourceDiagnostics", [])}


def reverse_actions(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    binary = Path(args.binary)
    if args.build:
        build_candidate(binary)
    if args.install_coder:
        install_coder_candidate(args)

    session = f"foreman-smoke-{os.getpid()}"
    run(["tmux", "new-session", "-d", "-s", session, "sh"], timeout=30)
    pane_id = run(["tmux", "display-message", "-p", "-t", session, "#{pane_id}"], timeout=30).stdout.strip()
    ready_file = artifact_dir / "companion-ready.txt"
    remote_port = 46000 + (os.getpid() % 1000)
    remote_dir = args.coder_workdir.rstrip("/")
    remote_config = f"{remote_dir}/coder-companion.toml"
    server: subprocess.Popen[str] | None = None
    tunnel: subprocess.Popen[str] | None = None
    try:
        server = subprocess.Popen(
            [
                str(binary),
                "companion",
                "serve",
                "--bind",
                "127.0.0.1:0",
                "--allow-send",
                "--ready-file",
                str(ready_file),
                "--json",
            ],
            cwd=str(REPO),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        deadline = time.time() + 10
        while time.time() < deadline and not ready_file.exists():
            if server.poll() is not None:
                out, err = server.communicate(timeout=1)
                raise RuntimeError(f"companion server exited early\nSTDOUT:\n{out}\nSTDERR:\n{err}")
            time.sleep(0.05)
        endpoint = ready_file.read_text().strip()
        local_port = endpoint.rsplit(":", 1)[1]
        tunnel = subprocess.Popen(
            [
                "ssh",
                "-N",
                "-o",
                "ControlMaster=no",
                "-o",
                "ControlPath=none",
                "-o",
                "ExitOnForwardFailure=yes",
                "-R",
                f"127.0.0.1:{remote_port}:127.0.0.1:{local_port}",
                args.coder_host,
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        deadline = time.time() + 15
        while time.time() < deadline:
            if tunnel.poll() is not None:
                out, err = tunnel.communicate(timeout=1)
                raise RuntimeError(f"reverse tunnel exited early\nSTDOUT:\n{out}\nSTDERR:\n{err}")
            probe = subprocess.run(
                [
                    "ssh",
                    args.coder_host,
                    "python3",
                    "-c",
                    f"import socket; s=socket.create_connection(('127.0.0.1',{remote_port}), 1); s.close()",
                ],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=5,
            )
            if probe.returncode == 0:
                break
            time.sleep(0.25)
        else:
            raise TimeoutError(f"reverse tunnel remote port {remote_port} did not become reachable")
        run(["ssh", args.coder_host, f"mkdir -p {shlex.quote(remote_dir)}"], timeout=60)
        local_config = artifact_dir / "coder-companion.toml"
        write(local_config, base_config(companion_endpoint=f"127.0.0.1:{remote_port}"))
        run(["scp", str(local_config), f"{args.coder_host}:{remote_config}"], timeout=120)
        agents = json.loads(
            run(
                [
                    "ssh",
                    args.coder_host,
                    f"{shlex.quote(args.coder_foreman)} --config-file {shlex.quote(remote_config)} agents --json --all-panes",
                ],
                timeout=120,
            ).stdout
        )
        counts = assert_source_counts(agents, {"local", "mac-live"})
        focus = json.loads(
            run(
                [
                    "ssh",
                    args.coder_host,
                    f"{shlex.quote(args.coder_foreman)} --config-file {shlex.quote(remote_config)} --source mac-live focus --pane {shlex.quote(pane_id)} --json",
                ],
                timeout=120,
            ).stdout
        )
        marker = f"foreman-smoke-{os.getpid()}"
        send = json.loads(
            run(
                [
                    "ssh",
                    args.coder_host,
                    f"{shlex.quote(args.coder_foreman)} --config-file {shlex.quote(remote_config)} --source mac-live send --pane {shlex.quote(pane_id)} --text {shlex.quote('echo ' + marker)} --json",
                ],
                timeout=120,
            ).stdout
        )
        time.sleep(0.5)
        capture = run(["tmux", "capture-pane", "-p", "-t", pane_id], timeout=30).stdout
        if marker not in capture:
            raise AssertionError(f"send marker {marker} not observed in local tmux pane capture")
        write(artifact_dir / "coder-live-agents-response.json", json.dumps(agents, indent=2) + "\n")
        write(artifact_dir / "focus-response.json", json.dumps(focus, indent=2) + "\n")
        write(artifact_dir / "send-response.json", json.dumps(send, indent=2) + "\n")
        write(artifact_dir / "tmux-capture.txt", capture)
        return {
            "endpoint": endpoint,
            "remoteEndpoint": f"127.0.0.1:{remote_port}",
            "paneId": pane_id,
            "counts": counts,
            "focusOk": focus.get("ok"),
            "sendOk": send.get("ok"),
            "marker": marker,
        }
    finally:
        if tunnel and tunnel.poll() is None:
            tunnel.terminate()
            try:
                tunnel.wait(timeout=5)
            except subprocess.TimeoutExpired:
                tunnel.kill()
        if server and server.poll() is None:
            server.terminate()
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()
        run(["tmux", "kill-session", "-t", session], timeout=30)


def full(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    results = {
        "snapshot": snapshot(args, artifact_dir / "snapshot"),
        "companionLocal": companion_local(args, artifact_dir / "companion-local"),
    }
    if args.coder:
        results["coderSnapshot"] = coder_snapshot(args, artifact_dir / "coder-snapshot")
        results["reverseActions"] = reverse_actions(args, artifact_dir / "reverse-actions")
    return results


def main() -> int:
    parser = argparse.ArgumentParser(description="Foreman source companion live smoke harness")
    parser.add_argument("scenario", choices=["snapshot", "companion-local", "coder-snapshot", "reverse-actions", "full"])
    parser.add_argument("--binary", default=str(REPO / "target" / "debug" / "foreman"))
    parser.add_argument("--build", action="store_true", default=True)
    parser.add_argument("--no-build", action="store_false", dest="build")
    parser.add_argument("--artifact-dir", default=".ai/validation/source-companion/latest")
    parser.add_argument("--coder", action="store_true", help="Include live Coder SSH checks in full scenario")
    parser.add_argument("--coder-host", default=DEFAULT_CODER_HOST)
    parser.add_argument("--coder-workdir", default="/tmp/foreman-source-companion-smoke")
    parser.add_argument("--coder-foreman", default="/home/discord/.cargo/bin/foreman-coder-source")
    parser.add_argument("--install-coder", action="store_true", help="Rsync this checkout to Coder and cargo-install it before live Coder checks")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    artifact_dir = Path(args.artifact_dir).resolve()
    if artifact_dir.exists():
        shutil.rmtree(artifact_dir)
    artifact_dir.mkdir(parents=True, exist_ok=True)

    try:
        if args.scenario == "snapshot":
            result = snapshot(args, artifact_dir)
        elif args.scenario == "companion-local":
            result = companion_local(args, artifact_dir)
        elif args.scenario == "coder-snapshot":
            result = coder_snapshot(args, artifact_dir)
        elif args.scenario == "reverse-actions":
            result = reverse_actions(args, artifact_dir)
        else:
            result = full(args, artifact_dir)
        summary = {"ok": True, "scenario": args.scenario, "artifactDir": str(artifact_dir), "result": result}
        write(artifact_dir / "summary.json", json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2) if args.json else f"ok scenario={args.scenario} artifactDir={artifact_dir}")
        return 0
    except Exception as exc:  # noqa: BLE001 - smoke harness should capture all failures.
        summary = {"ok": False, "scenario": args.scenario, "artifactDir": str(artifact_dir), "error": str(exc)}
        write(artifact_dir / "summary.json", json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
