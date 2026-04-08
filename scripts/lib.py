"""Shared helpers for agent-scaffold mise task scripts.

Import in any task script with:

    import os, sys
    sys.path.insert(0, os.path.join(os.environ["MISE_PROJECT_ROOT"], "scripts"))
    from lib import ...
"""

from __future__ import annotations

import os
import subprocess
import sys
import tomllib
from collections.abc import Callable
from pathlib import Path

# ── Project root ──────────────────────────────────────────────────────────────

PROJECT_ROOT = Path(os.environ.get("MISE_PROJECT_ROOT", "."))

# ── Logging ───────────────────────────────────────────────────────────────────

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


# ── Configuration ─────────────────────────────────────────────────────────────


def get_stack() -> str:
    return os.environ.get("SCAFFOLD_PROJECT_STACK", "python")


def get_shape() -> str:
    return os.environ.get("SCAFFOLD_PROJECT_SHAPE", "single")


def get_project_name() -> str:
    return os.environ.get("SCAFFOLD_PROJECT_NAME", "agent-scaffold")


# ── Subprocess helper ─────────────────────────────────────────────────────────


def run(cmd: list[str], cwd: Path | None = None) -> None:
    """Run *cmd* in *cwd* (defaults to PROJECT_ROOT); exit on failure."""
    result = subprocess.run(cmd, cwd=cwd or PROJECT_ROOT)  # noqa: S603
    if result.returncode != 0:
        sys.exit(result.returncode)


# ── Apps workspace helpers ────────────────────────────────────────────────────


def get_modules() -> list[dict]:
    """Parse workspace.toml and return a list of module dicts."""
    ws = PROJECT_ROOT / "workspace.toml"
    if not ws.exists():
        log_error("workspace.toml not found (required for apps shape)")
        sys.exit(1)
    with ws.open("rb") as f:
        data = tomllib.load(f)
    return [
        {
            "name": name,
            "path": info.get("path", f"apps/{name}"),
            "kind": info.get("kind", "python"),
            "role": info.get("role", "app"),
        }
        for name, info in data.get("modules", {}).items()
    ]


def run_per_module(
    task_name: str,
    fn: Callable[[dict, Path], None],
) -> None:
    """Call *fn(mod, cwd)* for every module in workspace.toml.

    Collects failures rather than stopping on the first error, then exits
    non-zero if any module failed (matching the bash run_per_module behavior).
    """
    failed: list[str] = []
    for mod in get_modules():
        cwd = PROJECT_ROOT / mod["path"]
        log_step(f"[{mod['name']}] {task_name}")
        try:
            fn(mod, cwd)
            log_ok(f"[{mod['name']}] {task_name}")
        except SystemExit as exc:
            rc = exc.code if isinstance(exc.code, int) else 1
            if rc != 0:
                log_error(f"[{mod['name']}] {task_name} failed")
                failed.append(mod["name"])
            else:
                log_ok(f"[{mod['name']}] {task_name}")

    if failed:
        log_error(f"Failed modules: {', '.join(failed)}")
        sys.exit(1)


# ── Stack dispatch ────────────────────────────────────────────────────────────


def dispatch_stack(fn_map: dict[str, Callable[[], None]]) -> None:
    """Call the function in *fn_map* keyed by the current stack.

    If the stack has no entry (e.g. rust, web are stubs), emits a warning
    and returns instead of failing.
    """
    stack = get_stack()
    fn = fn_map.get(stack)
    if fn is None:
        log_warn(f"{stack} stack: not yet implemented")
        return
    fn()


def dispatch_module(
    fn_map: dict[str, Callable[[Path], None]],
) -> Callable[[dict, Path], None]:
    """Return a callback for run_per_module that dispatches by module kind.

    Example::

        run_per_module("fmt", dispatch_module({"python": fmt_python, "go": fmt_go}))
    """

    def _dispatch(mod: dict, cwd: Path) -> None:
        fn = fn_map.get(mod["kind"])
        if fn is None:
            log_warn(f"{mod['kind']} stack: not yet implemented")
            return
        fn(cwd)

    return _dispatch
