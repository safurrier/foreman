#!/usr/bin/env python3
"""JSON-only receipt rendering shared by Foreman's unattended entrypoints."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Dict, List, Optional, TypedDict, Union, cast

SCHEMA_VERSION = 1
JsonValue = Union[
    None, bool, int, float, str, List["JsonValue"], Dict[str, "JsonValue"]
]
JsonObject = Dict[str, JsonValue]


class Command(TypedDict):
    argv: list[str]
    cwd: str


class DoctorFinding(TypedDict):
    id: str
    severity: str
    area: str
    provider: str | None
    pane_id: str | None
    repo_path: str | None
    summary: str
    detail: str | None
    evidence: list[str]
    next_step: str | None


class DoctorFix(TypedDict):
    provider: str | None
    path: str
    status: str
    message: str
    preview: str | None


class ReceiptMessage(TypedDict):
    status: str
    stage: str
    code: str
    message: str
    next_command: Command | None
    finding: DoctorFinding | None


class DoctorReport(TypedDict):
    repo_path: str | None
    findings: list[DoctorFinding]
    fixes: list[DoctorFix]


DOCTOR_FINDING_FIELDS = frozenset(DoctorFinding.__required_keys__)
DOCTOR_FIX_FIELDS = frozenset(DoctorFix.__required_keys__)
DOCTOR_REPORT_FIELDS = frozenset(DoctorReport.__required_keys__)
HARNESS_KINDS = {"ClaudeCode", "CodexCli", "Pi", "GeminiCli", "OpenCode"}


def command(root: Path, *argv: str) -> Command:
    return {"argv": list(argv), "cwd": str(root)}


def commands(root: Path) -> dict[str, Command]:
    return {
        "setup": command(root, str(root / ".agents" / "setup")),
        "doctor": command(
            root, "foreman", "--doctor", "--doctor-json", "--repo", str(root)
        ),
        "install_local": command(root, "mise", "run", "install-local"),
        "focused": command(root, "python3", "tests/agent_entrypoints_test.py"),
        "broad": command(root, "mise", "run", "check"),
        "dev": command(root, "mise", "run", "dev"),
    }


def env_path(name: str, root: Path) -> Path | None:
    if name not in os.environ:
        return None
    path = Path(os.environ[name])
    return path if path.is_absolute() else root / path


def optional_env_path(name: str) -> Path | None:
    value = os.environ.get(name)
    return Path(value) if value else None


def default_log_dir(root: Path) -> Path:
    if path := env_path("FOREMAN_LOG_DIR", root):
        return path
    if path := env_path("XDG_STATE_HOME", root):
        return path / "foreman" / "logs"
    home = env_path("HOME", root) or Path.home()
    return home / ".local" / "state" / "foreman" / "logs"


def load_json(path: Path | None) -> tuple[JsonValue | None, str | None]:
    if path is None:
        return None, None
    try:
        with path.open(encoding="utf-8") as file:
            return cast(JsonValue, json.load(file)), None
    except OSError as error:
        return None, str(error)
    except json.JSONDecodeError as error:
        return None, str(error)


def json_object(value: JsonValue) -> JsonObject | None:
    return value if isinstance(value, dict) else None


def validate_finding(value: JsonValue, _index: int) -> DoctorFinding | None:
    finding = json_object(value)
    if finding is None or set(finding) != DOCTOR_FINDING_FIELDS:
        return None
    required_strings = ("id", "severity", "area", "summary")
    if any(not isinstance(finding[name], str) for name in required_strings):
        return None
    if finding["severity"] not in {"ok", "info", "warn", "error"}:
        return None
    if finding["area"] not in {"machine", "config", "repo", "runtime"}:
        return None
    provider = finding["provider"]
    if provider is not None and (
        not isinstance(provider, str) or provider not in HARNESS_KINDS
    ):
        return None
    if not isinstance(finding["evidence"], list):
        return None
    if any(
        not isinstance(item, str) for item in cast(list[JsonValue], finding["evidence"])
    ):
        return None
    for name in ("pane_id", "repo_path", "detail", "next_step"):
        if finding[name] is not None and not isinstance(finding[name], str):
            return None
    return cast(DoctorFinding, finding)


def validate_fix(value: JsonValue, _index: int) -> DoctorFix | None:
    fix = json_object(value)
    if fix is None or set(fix) != DOCTOR_FIX_FIELDS:
        return None
    provider = fix["provider"]
    if provider is not None and (
        not isinstance(provider, str) or provider not in HARNESS_KINDS
    ):
        return None
    if not all(isinstance(fix[name], str) for name in ("path", "status", "message")):
        return None
    if fix["status"] not in {"planned", "written", "unchanged", "skipped"}:
        return None
    if fix["preview"] is not None and not isinstance(fix["preview"], str):
        return None
    return cast(DoctorFix, fix)


def validate_doctor_report(
    value: JsonValue | None,
) -> tuple[DoctorReport | None, str | None]:
    report = json_object(value) if value is not None else None
    if report is None:
        return None, "doctor JSON must be an object with repo_path, findings, and fixes"
    if set(report) != DOCTOR_REPORT_FIELDS:
        return None, "doctor JSON must contain exactly repo_path, findings, and fixes"
    if not isinstance(report["repo_path"], (str, type(None))):
        return None, "doctor JSON field repo_path must be a string or null"
    findings_value = report.get("findings")
    fixes_value = report.get("fixes")
    if not isinstance(findings_value, list) or not isinstance(fixes_value, list):
        return None, "doctor JSON fields findings and fixes must be arrays"
    findings: list[DoctorFinding] = []
    for index, item in enumerate(cast(list[JsonValue], findings_value)):
        finding = validate_finding(item, index)
        if finding is None:
            return (
                None,
                f"doctor JSON finding at index {index} does not match the known finding schema",
            )
        findings.append(finding)
    fixes: list[DoctorFix] = []
    for index, item in enumerate(cast(list[JsonValue], fixes_value)):
        fix = validate_fix(item, index)
        if fix is None:
            return (
                None,
                f"doctor JSON fix at index {index} does not match the known fix schema",
            )
        fixes.append(fix)
    return {
        "repo_path": cast(Optional[str], report["repo_path"]),
        "findings": findings,
        "fixes": fixes,
    }, None


def message(
    status: str,
    stage: str,
    code: str,
    text: str,
    next_command: Command | None,
    finding: DoctorFinding | None = None,
) -> ReceiptMessage:
    return {
        "status": status,
        "stage": stage,
        "code": code,
        "message": text,
        "next_command": next_command,
        "finding": finding,
    }


def doctor_messages(
    report: DoctorReport | None, command_map: dict[str, Command]
) -> tuple[list[ReceiptMessage], list[ReceiptMessage]]:
    warnings: list[ReceiptMessage] = []
    errors: list[ReceiptMessage] = []
    if report is None:
        return warnings, errors
    for finding in report["findings"]:
        if finding["severity"] == "warn":
            warnings.append(
                message(
                    "warning",
                    "doctor",
                    "doctor-finding",
                    "Foreman doctor reported a warning",
                    command_map["doctor"],
                    finding,
                )
            )
        elif finding["severity"] == "error":
            errors.append(
                message(
                    "error",
                    "doctor",
                    "doctor-finding",
                    "Foreman doctor reported a blocking finding",
                    command_map["setup"],
                    finding,
                )
            )
    return warnings, errors


def shared(
    root: Path, report: DoctorReport | None
) -> tuple[dict[str, object], list[ReceiptMessage], list[ReceiptMessage]]:
    log_dir = default_log_dir(root)
    latest = log_dir / "latest.log"
    command_map = commands(root)
    warnings, errors = doctor_messages(report, command_map)
    return (
        {
            "commands": command_map,
            "logs": {
                "directory": str(log_dir),
                "latest": str(latest) if latest.is_file() else None,
            },
        },
        warnings,
        errors,
    )


def doctor_report() -> tuple[DoctorReport | None, str | None]:
    value, parse_error = load_json(optional_env_path("FOREMAN_AGENT_DOCTOR_FILE"))
    if parse_error:
        return None, f"doctor JSON was not parseable: {parse_error}"
    if value is None:
        return None, None
    return validate_doctor_report(value)


def setup(root: Path) -> dict[str, object]:
    setup_status = os.environ["FOREMAN_AGENT_SETUP_STATUS"]
    setup_exit = int(os.environ["FOREMAN_AGENT_SETUP_EXIT"])
    doctor_status = os.environ["FOREMAN_AGENT_DOCTOR_STATUS"]
    doctor_exit = int(os.environ["FOREMAN_AGENT_DOCTOR_EXIT"])
    report, report_error = doctor_report()
    extra, warnings, errors = shared(root, report)
    command_map = cast(dict[str, Command], extra["commands"])
    if setup_status == "failed":
        errors.append(
            message(
                "error",
                "setup",
                "setup-failed",
                "mise run setup failed",
                command_map["setup"],
            )
        )
    if doctor_status in {"blocked", "failed"}:
        next_command = (
            command_map["install_local"] if doctor_exit == 127 else command_map["setup"]
        )
        errors.append(
            message(
                "error",
                "doctor",
                "doctor-failed",
                "Foreman strict doctor did not pass",
                next_command,
            )
        )
    if report_error:
        errors.append(
            message(
                "error",
                "doctor",
                "doctor-schema-invalid",
                report_error,
                command_map["doctor"],
            )
        )
    return {
        "schema": "foreman.agent.setup-receipt",
        "version": SCHEMA_VERSION,
        "ready": setup_status == "passed"
        and doctor_status == "passed"
        and report_error is None,
        "setup": {"status": setup_status, "exit_code": setup_exit},
        "doctor": {"status": doctor_status, "exit_code": doctor_exit, "report": report},
        "warnings": warnings,
        "errors": errors,
        **extra,
    }


def resume(root: Path) -> dict[str, object]:
    doctor_status = os.environ["FOREMAN_AGENT_DOCTOR_STATUS"]
    doctor_exit = int(os.environ["FOREMAN_AGENT_DOCTOR_EXIT"])
    report, report_error = doctor_report()
    hk_value, hk_error = load_json(optional_env_path("FOREMAN_AGENT_HK_FILE"))
    extra, warnings, errors = shared(root, report)
    command_map = cast(dict[str, Command], extra["commands"])
    repo_status = os.environ.get("FOREMAN_AGENT_REPO_STATUS", "available")
    if repo_status != "available":
        warnings.append(
            message(
                "warning",
                "repo",
                "repo-state-unavailable",
                "Git repository state could not be collected",
                None,
            )
        )
    hk_available = os.environ.get("FOREMAN_AGENT_HK_AVAILABLE") == "1"
    if not hk_available:
        warnings.append(
            message(
                "warning",
                "hk",
                "hk-unavailable",
                "hk is not installed; lifecycle status is unavailable",
                None,
            )
        )
    elif hk_error:
        warnings.append(
            message(
                "warning",
                "hk",
                "hk-status-unavailable",
                f"hk status is unavailable: {hk_error}",
                command_map["setup"],
            )
        )
    if cast(dict[str, object], extra["logs"])["latest"] is None:
        warnings.append(
            message(
                "warning",
                "logs",
                "latest-log-unavailable",
                "latest Foreman log is not available",
                command_map["doctor"],
            )
        )
    if report_error:
        errors.append(
            message(
                "error",
                "doctor",
                "doctor-schema-invalid",
                report_error,
                command_map["doctor"],
            )
        )
    if doctor_status in {"failed", "unavailable"}:
        next_command = (
            command_map["install_local"] if doctor_exit == 127 else command_map["setup"]
        )
        errors.append(
            message(
                "error",
                "doctor",
                "doctor-unavailable",
                "current doctor findings could not be collected",
                next_command,
            )
        )
    branch = os.environ.get("FOREMAN_AGENT_BRANCH") or None
    return {
        "schema": "foreman.agent.resume-report",
        "version": SCHEMA_VERSION,
        "repo": {
            "status": repo_status,
            "root": str(root),
            "branch": branch,
            "detached": os.environ.get("FOREMAN_AGENT_DETACHED") == "1",
            "dirty": os.environ.get("FOREMAN_AGENT_DIRTY") == "1",
        },
        "doctor": {"status": doctor_status, "exit_code": doctor_exit, "report": report},
        "optional_tools": {
            "hk": {
                "available": hk_available,
                "status": hk_value if hk_available else None,
            }
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
    root = args.root.resolve()
    os.environ["FOREMAN_AGENT_ROOT"] = str(root)
    receipt = setup(root) if args.kind == "setup" else resume(root)
    print(json.dumps(receipt, sort_keys=True, separators=(",", ":")))
    errors = cast(list[ReceiptMessage], receipt["errors"])
    if any(item["code"] == "doctor-schema-invalid" for item in errors):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
