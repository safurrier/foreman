#!/usr/bin/env python3
"""JSON-only receipt rendering shared by Foreman's unattended entrypoints."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1


def env_path(name: str) -> Path | None:
    value = os.environ.get(name)
    return Path(value) if value else None


def default_log_dir() -> Path:
    if value := os.environ.get("FOREMAN_LOG_DIR"):
        return Path(value)
    if value := os.environ.get("XDG_STATE_HOME"):
        return Path(value) / "foreman" / "logs"
    return Path.home() / ".local" / "state" / "foreman" / "logs"


def load_json(path: Path | None) -> tuple[Any | None, str | None]:
    if path is None:
        return None, None
    try:
        with path.open(encoding="utf-8") as file:
            return json.load(file), None
    except (OSError, json.JSONDecodeError) as error:
        return None, str(error)


def findings(report: Any | None) -> tuple[list[Any], list[Any]]:
    if not isinstance(report, dict):
        return [], []
    values = report.get("findings", [])
    if not isinstance(values, list):
        return [], []
    warnings = [item for item in values if isinstance(item, dict) and item.get("severity") == "warn"]
    errors = [item for item in values if isinstance(item, dict) and item.get("severity") == "error"]
    return warnings, errors


def commands(root: Path) -> dict[str, str]:
    quoted_root = str(root)
    return {
        "setup": ".agents/setup",
        "doctor": f"foreman --doctor --doctor-json --repo {quoted_root}",
        "focused": "python3 tests/agent_entrypoints_test.py",
        "broad": "mise run check",
        "dev": "mise run dev",
    }


def shared(root: Path, report: Any | None) -> tuple[dict[str, Any], list[Any], list[Any]]:
    log_dir = default_log_dir()
    latest = log_dir / "latest.log"
    warnings, errors = findings(report)
    return (
        {
            "commands": commands(root),
            "logs": {"directory": str(log_dir), "latest": str(latest) if latest.is_file() else None},
        },
        warnings,
        errors,
    )


def setup(root: Path) -> dict[str, Any]:
    setup_status = os.environ["FOREMAN_AGENT_SETUP_STATUS"]
    setup_exit = int(os.environ["FOREMAN_AGENT_SETUP_EXIT"])
    doctor_status = os.environ["FOREMAN_AGENT_DOCTOR_STATUS"]
    doctor_exit = int(os.environ["FOREMAN_AGENT_DOCTOR_EXIT"])
    report, report_error = load_json(env_path("FOREMAN_AGENT_DOCTOR_FILE"))
    extra, warnings, errors = shared(root, report)
    if setup_status == "failed":
        errors.append({"stage": "setup", "message": "mise run setup failed"})
    if doctor_status in {"blocked", "failed"}:
        errors.append({"stage": "doctor", "message": "foreman strict doctor did not pass"})
    if report_error:
        errors.append({"stage": "doctor", "message": f"doctor JSON was not parseable: {report_error}"})
    return {
        "schema": "foreman.agent.setup-receipt",
        "version": SCHEMA_VERSION,
        "ready": setup_status == "passed" and doctor_status == "passed" and report_error is None,
        "setup": {"status": setup_status, "exit_code": setup_exit},
        "doctor": {
            "status": doctor_status,
            "exit_code": doctor_exit,
            "report": report,
        },
        "warnings": warnings,
        "errors": errors,
        **extra,
    }


def resume(root: Path) -> dict[str, Any]:
    doctor_status = os.environ["FOREMAN_AGENT_DOCTOR_STATUS"]
    doctor_exit = int(os.environ["FOREMAN_AGENT_DOCTOR_EXIT"])
    report, report_error = load_json(env_path("FOREMAN_AGENT_DOCTOR_FILE"))
    hk_status, hk_error = load_json(env_path("FOREMAN_AGENT_HK_FILE"))
    extra, doctor_warnings, doctor_errors = shared(root, report)
    warnings: list[Any] = doctor_warnings
    errors: list[Any] = doctor_errors
    hk_available = os.environ.get("FOREMAN_AGENT_HK_AVAILABLE") == "1"
    if not hk_available:
        warnings.append("hk is not installed; lifecycle status is unavailable")
    elif hk_error:
        warnings.append(f"hk status is unavailable: {hk_error}")
    if extra["logs"]["latest"] is None:
        warnings.append("latest Foreman log is not available")
    if report_error:
        errors.append({"stage": "doctor", "message": f"doctor JSON was not parseable: {report_error}"})
    if doctor_status in {"failed", "unavailable"}:
        errors.append({"stage": "doctor", "message": "current doctor findings could not be collected"})
    branch = os.environ.get("FOREMAN_AGENT_BRANCH") or None
    return {
        "schema": "foreman.agent.resume-report",
        "version": SCHEMA_VERSION,
        "repo": {
            "root": str(root),
            "branch": branch,
            "detached": os.environ.get("FOREMAN_AGENT_DETACHED") == "1",
            "dirty": os.environ.get("FOREMAN_AGENT_DIRTY") == "1",
        },
        "doctor": {"status": doctor_status, "exit_code": doctor_exit, "report": report},
        "optional_tools": {
            "hk": {"available": hk_available, "status": hk_status if hk_available else None},
        },
        "warnings": warnings,
        "errors": errors,
        **extra,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("kind", choices=("setup", "resume"))
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    receipt = setup(args.root) if args.kind == "setup" else resume(args.root)
    print(json.dumps(receipt, sort_keys=True, separators=(",", ":")))


if __name__ == "__main__":
    main()
