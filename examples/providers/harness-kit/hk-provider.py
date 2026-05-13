#!/usr/bin/env python3
"""Foreman extension provider for Harness Kit repo lifecycle state."""

from __future__ import annotations

import argparse
import json
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

DATE_PREFIX = re.compile(r"^\d{4}-\d{2}-\d{2}-\d{6}-")
TRANSCRIPT_PATTERN = "ev_*.transcript.log"
HK_TIMEOUT_SECONDS = 12


def emit(cards: list[dict[str, Any]]) -> None:
    print(json.dumps({"cards": cards}, separators=(",", ":")))


def unavailable(message: str) -> None:
    emit(
        [
            {
                "id": "hk",
                "title": "Harness Kit",
                "status": "unavailable",
                "statusLabel": "UNAVAILABLE",
                "summary": message,
                "rows": [{"label": "Provider", "value": message, "status": "fail"}],
                "actions": [],
            }
        ]
    )


def find_hk() -> str | None:
    path_match = shutil.which("hk")
    if path_match is not None:
        return path_match

    candidates = [
        Path.home() / ".local" / "bin" / "hk",
        Path.home() / ".cargo" / "bin" / "hk",
        Path("/opt/homebrew/bin/hk"),
        Path("/usr/local/bin/hk"),
    ]
    for candidate in candidates:
        if candidate.is_file():
            return str(candidate)
    return None


def run_hk(hk: str, repo: Path, *args: str) -> dict[str, Any] | None:
    result = subprocess.run(
        [hk, *args, "--target", str(repo), "--json"],
        check=False,
        text=True,
        capture_output=True,
        timeout=HK_TIMEOUT_SECONDS,
    )
    if not result.stdout.strip():
        return None
    return json.loads(result.stdout)


def check_status(status: dict[str, Any], check_id: str) -> tuple[str, str]:
    for check in status.get("checks", []):
        if check.get("id") == check_id:
            return str(check.get("status", "info")), str(check.get("message", ""))
    return "info", "not reported"


def checks_with_prefix(status: dict[str, Any], prefix: str, *, check_status: str | None = None) -> list[str]:
    matches: list[str] = []
    for check in status.get("checks", []):
        check_id = str(check.get("id", ""))
        if not check_id.startswith(prefix):
            continue
        if check_status is not None and check.get("status") != check_status:
            continue
        matches.append(check_id.removeprefix(prefix))
    return matches


def normalize_row_status(value: str) -> str:
    if value == "pass":
        return "pass"
    if value == "fail":
        return "fail"
    return "info"


def normalize_active_work(value: Any) -> str:
    active_work = str(value or "none")
    return active_work if active_work else "none"


def short_work_id(active_work: str) -> str:
    if active_work == "none":
        return active_work
    return DATE_PREFIX.sub("", active_work)


def compact_message(message: str, fallback: str) -> str:
    if not message:
        return fallback
    if "validation evidence" in message and "stale" in message:
        return "stale for current diff"
    if "accepted review" in message and "does not cover" in message:
        return "missing for current diff"
    if "external-enough review recorded" in message:
        return "recorded"
    if "validation evidence with rationale recorded" in message:
        return "recorded"
    if "sync checkpoint stale" in message:
        return "needs sync"
    return message.split(". ", 1)[0]


def extract_command(action: str) -> str:
    # hk next_actions are human strings such as:
    # "validation: missing ...; run `hk validate --check fast-gate ...` using ..."
    run_match = re.search(r"run `([^`]+)`", action)
    if run_match:
        return run_match.group(1)
    command_match = re.search(r"`([^`]+)`", action)
    if command_match:
        return command_match.group(1)
    if ": " in action:
        return action.split(": ", 1)[1]
    return action


def target_hk_command(command: str, repo: Path) -> str:
    if not command.startswith("hk ") or " --target " in command:
        return command
    parts = command.split(" -- ", 1)
    targeted = f"{parts[0]} --target {shlex.quote(str(repo))}"
    if len(parts) == 2:
        return f"{targeted} -- {parts[1]}"
    return targeted


def choose_next_action(status: dict[str, Any], blocker: str, repo: Path) -> str | None:
    next_actions = [str(action) for action in status.get("next_actions") or []]
    if not next_actions:
        return None

    preferences: dict[str, list[str]] = {
        "validation": ["hk validate --check", "profile check", "validation:", "hk validate"],
        "review": ["hk review", "review:"],
        "sync": ["hk sync", "sync:"],
    }
    for needle in preferences.get(blocker, []):
        for action in next_actions:
            if needle in action:
                return target_hk_command(extract_command(action), repo)
    return target_hk_command(extract_command(next_actions[0]), repo)


def handoff_export(brief: dict[str, Any] | None) -> dict[str, Any]:
    if not brief:
        return {}
    value = brief.get("handoff_export")
    return value if isinstance(value, dict) else {}


def lifecycle_state(status: dict[str, Any], brief: dict[str, Any] | None) -> tuple[str, str, str]:
    active_work = normalize_active_work(status.get("active_work") or (brief or {}).get("active_work"))
    ready_status = str(status.get("ready_status") or status.get("status") or "unknown")

    if active_work == "none":
        return "idle", "NO WORK", "none"
    if ready_status == "ready":
        return "ready", "READY", "none"

    validation_status, _ = check_status(status, "validation")
    review_status, _ = check_status(status, "review")
    sync_status = str(status.get("sync_status") or (brief or {}).get("sync_status") or "unknown")
    failed_profile_checks = checks_with_prefix(status, "profile-check:", check_status="fail")
    failed_profile_reviews = checks_with_prefix(status, "profile-review:", check_status="fail")

    if validation_status == "fail" or failed_profile_checks:
        return "needs-validation", "NEEDS VALIDATION", "validation"
    if review_status == "fail" or failed_profile_reviews:
        return "needs-review", "NEEDS REVIEW", "review"
    if sync_status != "synced":
        return "needs-sync", "NEEDS SYNC", "sync"
    return "needs-attention", "NEEDS ATTENTION", "attention"


def latest_transcript(status: dict[str, Any], active_work: str) -> Path | None:
    state_dir = status.get("state_dir")
    if not state_dir or active_work == "none":
        return None
    artifacts_dir = Path(str(state_dir)) / "work" / active_work / "artifacts"
    if not artifacts_dir.exists():
        return None
    transcripts = sorted(
        artifacts_dir.glob(TRANSCRIPT_PATTERN),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    return transcripts[0] if transcripts else None


def append_open_action(actions: list[dict[str, str]], action_id: str, label: str, path: Path) -> None:
    if path.exists():
        actions.append(
            {
                "id": action_id,
                "label": label,
                "kind": "open-file",
                "value": str(path),
            }
        )


def append_copy_action(actions: list[dict[str, str]], action_id: str, label: str, value: str | None) -> None:
    if value:
        actions.append(
            {
                "id": action_id,
                "label": label,
                "kind": "copy",
                "value": value,
            }
        )


def export_row_status(export_state: str) -> str:
    if export_state == "fresh":
        return "pass"
    if export_state in {"stale", "invalid"}:
        return "fail"
    return "info"


def append_handoff_actions(actions: list[dict[str, str]], repo: Path, active_work: str, export: dict[str, Any]) -> None:
    if active_work == "none":
        return

    export_state = str(export.get("state") or "unknown")
    commands = export.get("commands") if isinstance(export.get("commands"), dict) else {}
    readme_path = str(export.get("readme_path") or "")
    readme_exists = bool(export.get("readme_exists"))

    if readme_exists and readme_path:
        actions.append(
            {
                "id": "open-handoff",
                "label": "Open stale handoff" if export_state == "stale" else "Open handoff",
                "kind": "open-file",
                "value": readme_path,
            }
        )
    elif not export:
        # Compatibility fallback for older HK versions that do not expose handoff_export.
        append_open_action(actions, "open-handoff", "Open handoff", repo / ".ai" / "hk" / active_work / "README.md")

    if export_state in {"missing", "stale", "invalid"}:
        append_copy_action(actions, "copy-export", "Copy export", commands.get("generate"))
    if export_state == "invalid":
        append_copy_action(actions, "copy-export-check", "Copy export check", commands.get("check"))
    append_copy_action(actions, "copy-handoff-preview", "Copy handoff preview", commands.get("preview"))


def start_command(repo: Path, brief: dict[str, Any] | None, status: dict[str, Any]) -> str | None:
    export = handoff_export(brief)
    commands = export.get("commands") if isinstance(export.get("commands"), dict) else {}
    command = commands.get("start") if commands else None
    if command:
        return str(command)
    return choose_next_action(status, "none", repo)


def build_card(repo: Path, brief: dict[str, Any] | None, status: dict[str, Any]) -> dict[str, Any]:
    active_work = normalize_active_work(status.get("active_work") or (brief or {}).get("active_work"))
    display_work = short_work_id(active_work)
    ready_status = str(status.get("ready_status") or status.get("status") or "unknown")
    sync_status = str(status.get("sync_status") or (brief or {}).get("sync_status") or "unknown")
    phase = str(status.get("phase") or "unknown")
    export = handoff_export(brief)
    export_state = str(export.get("state") or "unknown")

    card_status, status_label, blocker = lifecycle_state(status, brief)
    next_command = start_command(repo, brief, status) if card_status == "idle" else choose_next_action(status, blocker, repo)

    validation_status, validation_message = check_status(status, "validation")
    review_status, review_message = check_status(status, "review")
    failed_profile_checks = checks_with_prefix(status, "profile-check:", check_status="fail")
    passed_profile_checks = checks_with_prefix(status, "profile-check:", check_status="pass")
    failed_profile_reviews = checks_with_prefix(status, "profile-review:", check_status="fail")
    passed_profile_reviews = checks_with_prefix(status, "profile-review:", check_status="pass")

    validation_value = compact_message(validation_message, "not reported")
    if failed_profile_checks:
        validation_value = "needs " + ", ".join(failed_profile_checks[:3])
        if len(failed_profile_checks) > 3:
            validation_value += f", +{len(failed_profile_checks) - 3}"
    elif passed_profile_checks and validation_status == "pass":
        validation_value = "ok: " + ", ".join(passed_profile_checks[:3])

    review_value = compact_message(review_message, "not reported")
    if failed_profile_reviews:
        review_value = "needs " + ", ".join(failed_profile_reviews[:3])
    elif passed_profile_reviews and review_status == "pass":
        review_value = "ok: " + ", ".join(passed_profile_reviews[:3])

    if card_status == "idle":
        rows = []
    else:
        ready_row_status = "pass" if card_status == "ready" else "fail"
        sync_row_status = "pass" if sync_status == "synced" else "fail" if blocker == "sync" else "info"
        rows = [
            {"label": "Work", "value": display_work, "status": "info"},
            {"label": "Phase", "value": phase, "status": "info"},
            {"label": "Ready", "value": ready_status, "status": ready_row_status},
            {"label": "Sync", "value": sync_status, "status": sync_row_status},
            {"label": "Export", "value": export_state, "status": export_row_status(export_state)},
            {
                "label": "Validation",
                "value": validation_value,
                "status": "fail" if failed_profile_checks else normalize_row_status(validation_status),
            },
            {
                "label": "Review",
                "value": review_value,
                "status": "fail" if failed_profile_reviews else normalize_row_status(review_status),
            },
        ]

    actions: list[dict[str, str]] = []
    if next_command:
        actions.append(
            {
                "id": "copy-start" if card_status == "idle" else "copy-next",
                "label": "Copy start" if card_status == "idle" else "Copy next",
                "kind": "copy",
                "value": next_command,
            }
        )

    append_handoff_actions(actions, repo, active_work, export)

    if active_work != "none":
        transcript = latest_transcript(status, active_work)
        if transcript is not None:
            append_open_action(actions, "open-evidence", "Open evidence", transcript)
        state_dir = status.get("state_dir")
        if state_dir:
            append_open_action(actions, "open-work-dir", "Open work", Path(str(state_dir)) / "work" / active_work)

    if card_status != "idle":
        actions.append(
            {
                "id": "copy-status",
                "label": "Copy status",
                "kind": "copy",
                "value": f"hk status --target {shlex.quote(str(repo))} --json",
            }
        )

    if card_status == "ready":
        summary = "HK work is ready."
    elif card_status == "idle":
        summary = "No active HK work for this repo."
    elif next_command:
        summary = f"Next: {next_command}"
    else:
        summary = "HK work needs attention."

    return {
        "id": "hk",
        "title": "Harness Kit",
        "status": card_status,
        "statusLabel": status_label,
        "summary": summary,
        "rows": rows,
        "actions": actions,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Emit Foreman extension cards for Harness Kit state."
    )
    parser.add_argument("--repo", required=True, type=Path)
    args = parser.parse_args()

    hk = find_hk()
    if hk is None:
        unavailable("hk executable was not found; searched PATH, ~/.local/bin, ~/.cargo/bin, /opt/homebrew/bin, and /usr/local/bin")
        return 0

    try:
        brief = run_hk(hk, args.repo, "brief")
        status = run_hk(hk, args.repo, "status")
        if brief is None and status is None:
            emit([])
            return 0
        if status is None:
            status = {
                "active_work": (brief or {}).get("active_work") or "none",
                "sync_status": (brief or {}).get("sync_status") or "unknown",
                "ready_status": "not-started",
                "phase": "not-started",
                "checks": [],
                "next_actions": [],
            }
    except (OSError, subprocess.TimeoutExpired) as error:
        unavailable(f"failed to run hk: {error}")
        return 0
    except json.JSONDecodeError as error:
        unavailable(f"hk returned invalid JSON: {error}")
        return 0

    emit([build_card(args.repo, brief, status)])
    return 0


if __name__ == "__main__":
    sys.exit(main())
