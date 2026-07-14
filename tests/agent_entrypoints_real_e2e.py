#!/usr/bin/env python3
"""Real-tool disposable-clone E2E for Foreman's unattended entrypoints."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import TypedDict, cast

ROOT = Path(__file__).resolve().parents[1]


class Command(TypedDict):
    argv: list[str]
    cwd: str


def run(
    argv: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        argv,
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if check and result.returncode != 0:
        command = " ".join(argv)
        raise RuntimeError(
            f"command failed ({result.returncode}): {command}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result


def require_tool(name: str) -> str:
    executable = shutil.which(name)
    if executable is None:
        raise RuntimeError(f"required real-tool E2E dependency is missing: {name}")
    # Preserve shim/symlink argv[0] names (notably cargo -> rustup).
    return str(Path(executable).absolute())


def load_receipt(
    result: subprocess.CompletedProcess[str], label: str
) -> dict[str, object]:
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(
            f"{label} did not emit one JSON receipt\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        ) from error
    if not isinstance(value, dict):
        raise RuntimeError(f"{label} receipt must be a JSON object")
    return cast(dict[str, object], value)


def git_status(repo: Path, git: str) -> str:
    return run([git, "status", "--porcelain", "--untracked-files=all"], cwd=repo).stdout


def assert_command(receipt: dict[str, object], name: str, repo: Path) -> None:
    commands = cast(dict[str, object], receipt["commands"])
    value = cast(Command, commands[name])
    if value["cwd"] != str(repo.resolve()) or not value["argv"]:
        raise AssertionError(f"invalid structured command {name}: {value}")


def main() -> int:
    tools = {name: require_tool(name) for name in ("cargo", "git", "hk", "mise")}
    python = str(Path(sys.executable).resolve())

    run([tools["cargo"], "build", "--bin", "foreman"], cwd=ROOT)
    foreman = ROOT / "target" / "debug" / "foreman"
    if not foreman.is_file():
        raise RuntimeError(f"compiled Foreman binary is missing: {foreman}")

    temp_root = Path(tempfile.mkdtemp(prefix='foreman agent real e2e "quoted" '))
    clone = temp_root / 'foreman checkout "quoted"'
    home = temp_root / "home"
    bin_dir = temp_root / "bin"
    artifacts = temp_root / "artifacts"
    home.mkdir()
    bin_dir.mkdir()
    artifacts.mkdir()

    trusted = False
    passed = False
    try:
        run(
            [
                tools["git"],
                "clone",
                "--local",
                "--no-hardlinks",
                str(ROOT),
                str(clone),
            ]
        )
        # Exercise the working-tree entrypoint implementation before commit as well
        # as committed CI checkouts. Commit the overlay so drift checks start clean.
        shutil.copytree(ROOT / ".agents", clone / ".agents", dirs_exist_ok=True)
        run([tools["git"], "add", ".agents"], cwd=clone)
        run(
            [
                tools["git"],
                "-c",
                "user.name=Foreman E2E",
                "-c",
                "user.email=foreman-e2e@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "Overlay entrypoint under test",
            ],
            cwd=clone,
        )

        for name, target in {
            "foreman": foreman,
            "git": Path(tools["git"]),
            "hk": Path(tools["hk"]),
            "mise": Path(tools["mise"]),
            "python3": Path(python),
        }.items():
            (bin_dir / name).symlink_to(target)

        original_home = Path.home()
        env = {
            **os.environ,
            "HOME": str(home),
            "XDG_CONFIG_HOME": str(home / ".config"),
            "XDG_STATE_HOME": str(home / ".local" / "state"),
            "XDG_CACHE_HOME": str(home / ".cache"),
            # Reuse installed toolchains/caches while isolating config, state,
            # Foreman runtime files, and HK repository state.
            "CARGO_HOME": os.environ.get("CARGO_HOME", str(original_home / ".cargo")),
            "RUSTUP_HOME": os.environ.get(
                "RUSTUP_HOME", str(original_home / ".rustup")
            ),
            "MISE_DATA_DIR": os.environ.get(
                "MISE_DATA_DIR", str(original_home / ".local" / "share" / "mise")
            ),
            "PATH": f"{bin_dir}:/usr/bin:/bin",
        }

        run([tools["mise"], "trust", str(clone / "mise.toml")], env=env)
        trusted = True
        before = git_status(clone, tools["git"])

        setup_receipts: list[dict[str, object]] = []
        for attempt in (1, 2):
            result = run([str(clone / ".agents" / "setup")], cwd=Path("/"), env=env)
            (artifacts / f"setup-{attempt}.stderr").write_text(result.stderr)
            receipt = load_receipt(result, f"setup-{attempt}")
            (artifacts / f"setup-{attempt}.json").write_text(
                json.dumps(receipt, indent=2, sort_keys=True)
            )
            if receipt.get("ready") is not True:
                raise AssertionError(f"setup-{attempt} was not ready: {receipt}")
            doctor = cast(dict[str, object], receipt["doctor"])
            if doctor.get("status") != "passed":
                raise AssertionError(f"setup-{attempt} doctor did not pass: {doctor}")
            assert_command(receipt, "setup", clone)
            assert_command(receipt, "doctor", clone)
            setup_receipts.append(receipt)
            if git_status(clone, tools["git"]) != before:
                raise AssertionError(f"setup-{attempt} changed checkout state")

        if setup_receipts[0] != setup_receipts[1]:
            raise AssertionError("repeated setup receipts did not converge")

        run(
            [
                tools["hk"],
                "start",
                "real-entrypoint-e2e",
                "--target",
                ".",
                "--plan",
                "Disposable real-tool validation of setup and resume",
            ],
            cwd=clone,
            env=env,
        )
        before_resume = git_status(clone, tools["git"])
        resume_result = run([str(clone / ".agents" / "resume")], cwd=Path("/"), env=env)
        (artifacts / "resume.stderr").write_text(resume_result.stderr)
        resume = load_receipt(resume_result, "resume")
        (artifacts / "resume.json").write_text(
            json.dumps(resume, indent=2, sort_keys=True)
        )

        repo = cast(dict[str, object], resume["repo"])
        doctor = cast(dict[str, object], resume["doctor"])
        optional_tools = cast(dict[str, object], resume["optional_tools"])
        hk_status = cast(dict[str, object], optional_tools["hk"])
        if repo.get("status") != "available" or repo.get("dirty") is not False:
            raise AssertionError(f"resume reported incorrect repository state: {repo}")
        if doctor.get("status") != "ok":
            raise AssertionError(f"resume doctor did not pass: {doctor}")
        if hk_status.get("available") is not True or not isinstance(
            hk_status.get("status"), dict
        ):
            raise AssertionError(f"resume did not include real HK status: {hk_status}")
        hk_receipt = cast(dict[str, object], hk_status["status"])
        active_work = hk_receipt.get("active_work")
        if not isinstance(active_work, str) or not active_work.endswith(
            "-real-entrypoint-e2e"
        ):
            raise AssertionError(
                f"resume returned HK state for the wrong target/work item: {hk_receipt}"
            )
        if git_status(clone, tools["git"]) != before_resume:
            raise AssertionError("resume changed checkout state")

        print(
            json.dumps(
                {
                    "status": "passed",
                    "setupRuns": 2,
                    "setupReady": True,
                    "doctor": "passed",
                    "hk": "available",
                    "checkoutDrift": False,
                    "resumeReadOnly": True,
                },
                sort_keys=True,
            )
        )
        passed = True
        return 0
    finally:
        if trusted:
            run(
                [tools["mise"], "trust", "--untrust", str(clone / "mise.toml")],
                env=locals().get("env"),
                check=False,
            )
        if passed:
            shutil.rmtree(temp_root)
        else:
            print(f"real E2E artifacts preserved at {temp_root}", file=sys.stderr)


if __name__ == "__main__":
    raise SystemExit(main())
