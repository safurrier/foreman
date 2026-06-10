#!/usr/bin/env python3
"""Live Foreman source-companion smoke harness.

This is intentionally standard-library only so agents can run it on workstation and remote SSH hosts
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
DEFAULT_REMOTE_HOST = os.environ.get("FOREMAN_REMOTE_DEV_HOST", "remote-dev")


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


@dataclass
class TmuxFixture:
    socket: Path
    session: str
    pane_id: str


def create_tmux_fixture(artifact_dir: Path) -> TmuxFixture:
    socket = Path(tempfile.gettempdir()) / f"foreman-smoke-{os.getpid()}.sock"
    session = f"foreman-smoke-{os.getpid()}"
    run(["tmux", "-S", str(socket), "new-session", "-d", "-s", session, "sh"], timeout=30)
    pane_id = run(
        ["tmux", "-S", str(socket), "display-message", "-p", "-t", session, "#{pane_id}"],
        timeout=30,
    ).stdout.strip()
    return TmuxFixture(socket=socket, session=session, pane_id=pane_id)


def kill_tmux_fixture(fixture: TmuxFixture | None) -> None:
    if fixture is None:
        return
    subprocess.run(
        ["tmux", "-S", str(fixture.socket), "kill-server"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
        timeout=10,
    )
    try:
        fixture.socket.unlink()
    except FileNotFoundError:
        pass


def base_config(
    snapshot: Path | None = None,
    companion_endpoint: str | None = None,
    companion_token: str | None = None,
) -> str:
    chunks = ["[sources]\ndefault_scope = \"all\"\nquery_timeout_ms = 1000\n"]
    if snapshot:
        chunks.append(
            f"""
[sources.workstation]
kind = "snapshot"
label = "Workstation"
path = "{snapshot}"
"""
        )
    if companion_endpoint:
        chunks.append(
            f"""
[sources.workstation]
kind = "companion"
label = "Workstation"
endpoint = "{companion_endpoint}"
timeout_ms = 1000
allow_send = true
"""
        )
        if companion_token:
            chunks.append(f'token = "{companion_token}"\n')
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
    fixture: TmuxFixture | None = None
    try:
        fixture = create_tmux_fixture(artifact_dir)
        snapshot_path = artifact_dir / "workstation-source.agents.json"
        result = json_run(
            [
                str(binary),
                "--tmux-socket",
                str(fixture.socket),
                "sources",
                "snapshot",
                "--source-id",
                "workstation",
                "--output",
                str(snapshot_path),
                "--all-panes",
                "--json",
            ],
            timeout=120,
        )
        write(artifact_dir / "snapshot-result.json", json.dumps(result, indent=2) + "\n")
        return {"snapshot": result, "snapshotPath": str(snapshot_path)}
    finally:
        kill_tmux_fixture(fixture)


def companion_local(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    binary = Path(args.binary)
    if args.build:
        build_candidate(binary)
    fixture = create_tmux_fixture(artifact_dir)
    ready_file = artifact_dir / "companion-ready.txt"
    config_file = artifact_dir / "companion-client.toml"
    server = subprocess.Popen(
        [
            str(binary),
            "--tmux-socket",
            str(fixture.socket),
            "companion",
            "serve",
            "--bind",
            "127.0.0.1:0",
            "--allow-send",
            "--token",
            "smoke-token",
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
        write(config_file, base_config(companion_endpoint=endpoint, companion_token="smoke-token"))
        payload = json_run(
            [
                str(binary),
                "--tmux-socket",
                str(fixture.socket),
                "--config-file",
                str(config_file),
                "agents",
                "--json",
                "--all-panes",
            ],
            timeout=120,
        )
        counts = assert_source_counts(payload, {"local", "workstation"})
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
        kill_tmux_fixture(fixture)


def install_remote_candidate(args: argparse.Namespace) -> None:
    remote_dir = args.remote_workdir.rstrip("/") + "/repo"
    quoted_remote_dir = shlex.quote(remote_dir)
    run(["ssh", args.remote_host, f"rm -rf {quoted_remote_dir} && mkdir -p {quoted_remote_dir}"], timeout=120)
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
            f"{args.remote_host}:{remote_dir}/",
        ],
        timeout=300,
    )
    run(
        [
            "ssh",
            args.remote_host,
            f"cd {quoted_remote_dir} && cargo install --path . --locked --force",
        ],
        timeout=900,
    )


def remote_snapshot(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    binary = Path(args.binary)
    if args.build:
        build_candidate(binary)
    if args.install_remote:
        install_remote_candidate(args)
    snap = snapshot(args, artifact_dir)
    snapshot_path = Path(snap["snapshotPath"])
    remote_dir = args.remote_workdir.rstrip("/")
    remote_snapshot = f"{remote_dir}/workstation-source.agents.json"
    remote_config = f"{remote_dir}/remote-snapshot.toml"
    run(["ssh", args.remote_host, f"mkdir -p {shlex.quote(remote_dir)}"], timeout=60)
    run(["scp", str(snapshot_path), f"{args.remote_host}:{remote_snapshot}"], timeout=120)
    config_text = base_config(snapshot=Path(remote_snapshot))
    local_config = artifact_dir / "remote-snapshot.toml"
    write(local_config, config_text)
    run(["scp", str(local_config), f"{args.remote_host}:{remote_config}"], timeout=120)
    payload_text = run(
        [
            "ssh",
            args.remote_host,
            f"{shlex.quote(args.remote_foreman)} --config-file {shlex.quote(remote_config)} agents --sources all --json --all-panes",
        ],
        timeout=120,
    ).stdout
    payload = json.loads(payload_text)
    counts = assert_source_counts(payload, {"local", "workstation"})
    write(artifact_dir / "remote-agents-response.json", json.dumps(payload, indent=2) + "\n")
    return {"entryCount": len(payload.get("entries", [])), "bySource": counts, "diagnostics": payload.get("sourceDiagnostics", [])}


def reverse_actions(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    binary = Path(args.binary)
    if args.build:
        build_candidate(binary)
    if args.install_remote:
        install_remote_candidate(args)

    fixture = create_tmux_fixture(artifact_dir)
    pane_id = fixture.pane_id
    ready_file = artifact_dir / "companion-ready.txt"
    remote_port = 46000 + (os.getpid() % 1000)
    remote_dir = args.remote_workdir.rstrip("/")
    remote_config = f"{remote_dir}/remote-companion.toml"
    server: subprocess.Popen[str] | None = None
    tunnel: subprocess.Popen[str] | None = None
    try:
        server = subprocess.Popen(
            [
                str(binary),
                "--tmux-socket",
                str(fixture.socket),
                "companion",
                "serve",
                "--bind",
                "127.0.0.1:0",
                "--allow-send",
                "--token",
                "smoke-token",
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
                args.remote_host,
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        deadline = time.time() + 20
        probe_code = (
            "import json,socket; "
            f"s=socket.create_connection(('127.0.0.1',{remote_port}), 2); "
            "req={'protocolVersion':1,'requestId':'probe','token':'smoke-token','action':'agents','allPanes':False}; "
            "s.sendall((json.dumps(req)+'\\n').encode()); "
            "data=s.recv(4096); "
            "assert b'\"ok\":true' in data or b'\"ok\": true' in data, data; "
            "s.close()"
        )
        while time.time() < deadline:
            if tunnel.poll() is not None:
                out, err = tunnel.communicate(timeout=1)
                raise RuntimeError(f"reverse tunnel exited early\nSTDOUT:\n{out}\nSTDERR:\n{err}")
            probe = subprocess.run(
                ["ssh", args.remote_host, f"python3 -c {shlex.quote(probe_code)}"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=8,
            )
            if probe.returncode == 0:
                break
            time.sleep(0.5)
        else:
            raise TimeoutError(
                f"reverse tunnel remote port {remote_port} did not pass companion probe; last stderr={probe.stderr!r}"
            )
        run(["ssh", args.remote_host, f"mkdir -p {shlex.quote(remote_dir)}"], timeout=60)
        local_config = artifact_dir / "remote-companion.toml"
        write(local_config, base_config(companion_endpoint=f"127.0.0.1:{remote_port}", companion_token="smoke-token"))
        run(["scp", str(local_config), f"{args.remote_host}:{remote_config}"], timeout=120)
        agents = json.loads(
            run(
                [
                    "ssh",
                    args.remote_host,
                    f"{shlex.quote(args.remote_foreman)} --config-file {shlex.quote(remote_config)} agents --sources all --json --all-panes",
                ],
                timeout=120,
            ).stdout
        )
        counts = assert_source_counts(agents, {"local", "workstation"})
        focus = json.loads(
            run(
                [
                    "ssh",
                    args.remote_host,
                    f"{shlex.quote(args.remote_foreman)} --config-file {shlex.quote(remote_config)} --source workstation focus --pane {shlex.quote(pane_id)} --json",
                ],
                timeout=120,
            ).stdout
        )
        marker = f"foreman-smoke-{os.getpid()}"
        send = json.loads(
            run(
                [
                    "ssh",
                    args.remote_host,
                    f"{shlex.quote(args.remote_foreman)} --config-file {shlex.quote(remote_config)} --source workstation send --pane {shlex.quote(pane_id)} --text {shlex.quote('echo ' + marker)} --json",
                ],
                timeout=120,
            ).stdout
        )
        time.sleep(0.5)
        capture = run(["tmux", "-S", str(fixture.socket), "capture-pane", "-p", "-t", pane_id], timeout=30).stdout
        if marker not in capture:
            raise AssertionError(f"send marker {marker} not observed in local tmux pane capture")
        write(artifact_dir / "remote-live-agents-response.json", json.dumps(agents, indent=2) + "\n")
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
        kill_tmux_fixture(fixture)


def connect_ssh(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    binary = Path(args.binary)
    if args.build:
        build_candidate(binary)
    if args.install_remote:
        install_remote_candidate(args)

    fixture = create_tmux_fixture(artifact_dir)
    pane_id = fixture.pane_id
    marker_path = artifact_dir / "activation-marker.txt"
    remote_dir = args.remote_workdir.rstrip("/")
    remote_config = f"{remote_dir}/connect-ssh.toml"
    remote_port = 47000 + (os.getpid() % 1000)
    marker = f"foreman-connect-ssh-{os.getpid()}"
    activation_command = f"printf '%s\\n' {shlex.quote(marker + ':' + pane_id)} > {shlex.quote(str(marker_path))}"
    process: subprocess.Popen[str] | None = None
    try:
        run(["ssh", args.remote_host, f"mkdir -p {shlex.quote(remote_dir)}"], timeout=60)
        process = subprocess.Popen(
            [
                str(binary),
                "--tmux-socket",
                str(fixture.socket),
                "companion",
                "connect-ssh",
                args.remote_host,
                "--source-id",
                "workstation",
                "--label",
                "Workstation",
                "--remote-port",
                str(remote_port),
                "--allow-send",
                "--activation-command",
                activation_command,
                "--remote-foreman",
                args.remote_foreman,
                "--remote-config-file",
                remote_config,
                "--replace",
                "--json",
            ],
            cwd=str(REPO),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )

        agents: dict[str, Any] | None = None
        deadline = time.time() + 30
        while time.time() < deadline:
            if process.poll() is not None:
                out, err = process.communicate(timeout=1)
                raise RuntimeError(f"connect-ssh exited early\nSTDOUT:\n{out}\nSTDERR:\n{err}")
            probe = subprocess.run(
                [
                    "ssh",
                    args.remote_host,
                    f"{shlex.quote(args.remote_foreman)} --config-file {shlex.quote(remote_config)} agents --sources all --json --all-panes",
                ],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=10,
            )
            if probe.returncode == 0:
                payload = json.loads(probe.stdout)
                try:
                    assert_source_counts(payload, {"local", "workstation"})
                    agents = payload
                    break
                except AssertionError:
                    pass
            time.sleep(0.5)
        if agents is None:
            raise TimeoutError("connect-ssh did not configure a usable remote companion source")

        focus = json.loads(
            run(
                [
                    "ssh",
                    args.remote_host,
                    f"{shlex.quote(args.remote_foreman)} --config-file {shlex.quote(remote_config)} --source workstation focus --pane {shlex.quote(pane_id)} --json",
                ],
                timeout=120,
            ).stdout
        )
        deadline = time.time() + 5
        while time.time() < deadline and not marker_path.exists():
            time.sleep(0.1)
        if not marker_path.exists():
            raise AssertionError("activation marker was not written by companion-side activation command")
        activation_text = marker_path.read_text().strip()
        if marker not in activation_text or pane_id not in activation_text:
            raise AssertionError(f"activation marker mismatch: {activation_text!r}")

        send = json.loads(
            run(
                [
                    "ssh",
                    args.remote_host,
                    f"{shlex.quote(args.remote_foreman)} --config-file {shlex.quote(remote_config)} --source workstation send --pane {shlex.quote(pane_id)} --text {shlex.quote('echo ' + marker)} --json",
                ],
                timeout=120,
            ).stdout
        )
        time.sleep(0.5)
        capture = run(["tmux", "-S", str(fixture.socket), "capture-pane", "-p", "-t", pane_id], timeout=30).stdout
        if marker not in capture:
            raise AssertionError(f"send marker {marker} not observed in local tmux pane capture")

        counts = assert_source_counts(agents, {"local", "workstation"})
        write(artifact_dir / "remote-agents-response.json", json.dumps(agents, indent=2) + "\n")
        write(artifact_dir / "focus-response.json", json.dumps(focus, indent=2) + "\n")
        write(artifact_dir / "send-response.json", json.dumps(send, indent=2) + "\n")
        write(artifact_dir / "tmux-capture.txt", capture)
        return {
            "remoteHostLabel": "remote-dev",
            "remoteEndpoint": f"127.0.0.1:{remote_port}",
            "paneId": pane_id,
            "counts": counts,
            "focusOk": focus.get("ok"),
            "sendOk": send.get("ok"),
            "activationOk": True,
            "marker": marker,
        }
    finally:
        if process and process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                out, err = process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                out, err = process.communicate(timeout=5)
            write(artifact_dir / "connect-ssh.stdout.log", out)
            write(artifact_dir / "connect-ssh.stderr.log", err)
        kill_tmux_fixture(fixture)



def full(args: argparse.Namespace, artifact_dir: Path) -> dict[str, Any]:
    results = {
        "snapshot": snapshot(args, artifact_dir / "snapshot"),
        "companionLocal": companion_local(args, artifact_dir / "companion-local"),
    }
    if args.remote:
        results["remoteSnapshot"] = remote_snapshot(args, artifact_dir / "remote-snapshot")
        results["reverseActions"] = reverse_actions(args, artifact_dir / "reverse-actions")
        results["connectSsh"] = connect_ssh(args, artifact_dir / "connect-ssh")
    return results


def main() -> int:
    parser = argparse.ArgumentParser(description="Foreman source companion live smoke harness")
    parser.add_argument("scenario", choices=["snapshot", "companion-local", "remote-snapshot", "reverse-actions", "connect-ssh", "full"])
    parser.add_argument("--binary", default=str(REPO / "target" / "debug" / "foreman"))
    parser.add_argument("--build", action="store_true", default=True)
    parser.add_argument("--no-build", action="store_false", dest="build")
    parser.add_argument("--artifact-dir", default=".ai/validation/source-companion/latest")
    parser.add_argument("--remote", action="store_true", help="Include live remote SSH host checks in full scenario")
    parser.add_argument("--remote-host", default=DEFAULT_REMOTE_HOST)
    parser.add_argument("--remote-workdir", default="/tmp/foreman-source-companion-smoke")
    parser.add_argument("--remote-foreman", default="foreman")
    parser.add_argument("--install-remote", action="store_true", help="Rsync this checkout to the remote SSH host and cargo-install it before live checks")
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
        elif args.scenario == "remote-snapshot":
            result = remote_snapshot(args, artifact_dir)
        elif args.scenario == "reverse-actions":
            result = reverse_actions(args, artifact_dir)
        elif args.scenario == "connect-ssh":
            result = connect_ssh(args, artifact_dir)
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
