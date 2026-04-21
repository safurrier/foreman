"""Shared helpers for foreman mise task scripts."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

PROJECT_ROOT = Path(os.environ.get("MISE_PROJECT_ROOT", "."))
PROJECT_NAME = "foreman"
VALIDATION_ROOT = PROJECT_ROOT / ".ai" / "validation"
UX_VALIDATION_ROOT = VALIDATION_ROOT / "ux"
RELEASE_VALIDATION_ROOT = VALIDATION_ROOT / "release"

_B = "\033[1m"  # bold
_BLUE = "\033[34m"
_GREEN = "\033[32m"
_YELLOW = "\033[33m"
_RED = "\033[31m"
_R = "\033[0m"  # reset


def log_step(msg: str) -> None:
    print(f"{_B}{_BLUE}==>{_R} {_B}{msg}{_R}", flush=True)


def log_ok(msg: str) -> None:
    print(f"{_B}{_GREEN}  \u2713{_R} {msg}", flush=True)


def log_warn(msg: str) -> None:
    print(f"{_B}{_YELLOW}  !{_R} {msg}", file=sys.stderr, flush=True)


def log_error(msg: str) -> None:
    print(f"{_B}{_RED}  \u2717{_R} {msg}", file=sys.stderr, flush=True)


def run(cmd: list[str], cwd: Path | None = None) -> None:
    """Run *cmd* in *cwd* (defaults to PROJECT_ROOT); exit on failure."""
    result = subprocess.run(cmd, cwd=cwd or PROJECT_ROOT)  # noqa: S603
    if result.returncode != 0:
        sys.exit(result.returncode)


def validation_artifacts_required() -> bool:
    return os.environ.get("FOREMAN_REQUIRE_VALIDATION_ARTIFACTS") == "1" or os.environ.get(
        "CI"
    ) == "true"
