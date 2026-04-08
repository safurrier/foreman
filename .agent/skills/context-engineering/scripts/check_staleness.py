#!/usr/bin/env python3
"""Check staleness of AGENTS.md files relative to recent git activity.

Analyzes git history since the last AGENTS.md update and categorizes
changed files by impact level to determine if docs need updating.
"""

import datetime
import json
import re
import subprocess
import sys
from pathlib import Path

# File patterns that indicate HIGH impact changes
HIGH_IMPACT_PATTERNS = [
    r"(^|/)package\.json$",
    r"(^|/)pyproject\.toml$",
    r"(^|/)Cargo\.toml$",
    r"(^|/)go\.mod$",
    r"(^|/)go\.sum$",
    r"(^|/)pom\.xml$",
    r"(^|/)build\.gradle",
    r"(^|/)Makefile$",
    r"(^|/)Taskfile",
    r"(^|/)justfile$",
    r"(^|/)\.mise\.toml$",
    r"(^|/)\.github/workflows/",
    r"(^|/)\.gitlab-ci",
    r"(^|/)Jenkinsfile",
    r"(^|/)Dockerfile",
    r"(^|/)docker-compose",
    r"(^|/)\.pre-commit-config\.yaml$",
]

# File patterns that indicate MEDIUM impact changes
MEDIUM_IMPACT_PATTERNS = [
    r"(^|/)README",
    r"(^|/)CHANGELOG",
    r"(^|/)setup\.(py|cfg)$",
    r"(^|/)requirements.*\.txt$",
]


def run_git(args: list[str], cwd: Path) -> str:
    """Run a git command and return stdout."""
    result = subprocess.run(
        ["git", *args],
        cwd=cwd,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def get_last_update_commit(agents_path: Path) -> tuple[str, str]:
    """Get the commit hash and date of the last AGENTS.md modification."""
    cwd = agents_path.parent
    output = run_git(
        ["log", "-1", "--format=%H %aI", "--", agents_path.name],
        cwd=cwd,
    )
    if not output:
        return "", ""
    parts = output.split(" ", 1)
    return parts[0], parts[1] if len(parts) > 1 else ""


def get_commits_since(commit_hash: str, cwd: Path) -> list[dict]:
    """Get commits since the given hash with their changed files."""
    if not commit_hash:
        # No previous update — treat everything as new
        output = run_git(
            ["log", "--name-only", "--format=COMMIT:%H:%s", "-n", "100"],
            cwd=cwd,
        )
    else:
        output = run_git(
            [
                "log",
                "--name-only",
                "--format=COMMIT:%H:%s",
                f"{commit_hash}..HEAD",
            ],
            cwd=cwd,
        )

    if not output:
        return []

    commits = []
    current_commit = None

    for line in output.splitlines():
        if line.startswith("COMMIT:"):
            if current_commit:
                commits.append(current_commit)
            parts = line[7:].split(":", 1)
            current_commit = {
                "hash": parts[0],
                "message": parts[1] if len(parts) > 1 else "",
                "files": [],
            }
        elif line.strip() and current_commit:
            current_commit["files"].append(line.strip())

    if current_commit:
        commits.append(current_commit)

    return commits


def categorize_file(filepath: str) -> str:
    """Categorize a file change as HIGH, MEDIUM, or LOW impact."""
    for pattern in HIGH_IMPACT_PATTERNS:
        if re.search(pattern, filepath):
            return "HIGH"

    for pattern in MEDIUM_IMPACT_PATTERNS:
        if re.search(pattern, filepath):
            return "MEDIUM"

    # New or removed directories at the top level
    parts = filepath.split("/")
    if len(parts) <= 2:
        return "MEDIUM"

    return "LOW"


def detect_structural_changes(files: list[str], cwd: Path) -> bool:
    """Detect if directory structure significantly changed."""
    # Check if new top-level directories appeared
    current_dirs = {p.name for p in cwd.iterdir() if p.is_dir() and not p.name.startswith(".")}
    touched_dirs = {f.split("/")[0] for f in files if "/" in f}
    new_dirs = touched_dirs - current_dirs
    return len(new_dirs) > 0


def determine_recommendation(
    commits_since: int,
    high_count: int,
    medium_count: int,
    days_since: int,
) -> str:
    """Determine update recommendation based on staleness signals."""
    if commits_since == 0:
        return "skip"

    if days_since > 30 or commits_since > 50:
        return "full-rebuild"

    if high_count > 0:
        return "full-update"

    if medium_count > 0:
        return "partial-update"

    return "skip"


def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: check_staleness.py <path-to-AGENTS.md>")
        return 1

    agents_path = Path(sys.argv[1]).resolve()
    if not agents_path.exists():
        print(json.dumps({"error": f"{agents_path} not found", "recommendation": "full-rebuild"}))
        return 0

    cwd = agents_path.parent

    # Find git root
    git_root = run_git(["rev-parse", "--show-toplevel"], cwd=cwd)
    if not git_root:
        print(json.dumps({"error": "Not a git repository", "recommendation": "full-rebuild"}))
        return 0

    git_root_path = Path(git_root)

    last_hash, last_date = get_last_update_commit(agents_path)
    commits = get_commits_since(last_hash, git_root_path)

    # Collect all changed files and categorize
    all_files: list[str] = []
    for commit in commits:
        all_files.extend(commit["files"])

    unique_files = sorted(set(all_files))

    categorized = {"HIGH": [], "MEDIUM": [], "LOW": []}
    for f in unique_files:
        cat = categorize_file(f)
        categorized[cat].append(f)

    # Calculate days since last update
    days_since = 0
    if last_date:
        try:
            last_dt = datetime.datetime.fromisoformat(last_date)
            now = datetime.datetime.now(datetime.UTC)
            days_since = (now - last_dt).days
        except ValueError:
            days_since = -1

    recommendation = determine_recommendation(
        commits_since=len(commits),
        high_count=len(categorized["HIGH"]),
        medium_count=len(categorized["MEDIUM"]),
        days_since=days_since,
    )

    result = {
        "agents_file": str(agents_path),
        "last_updated": {"commit": last_hash, "date": last_date},
        "commits_since": len(commits),
        "days_since": days_since,
        "changed_files": {
            "high_impact": categorized["HIGH"],
            "medium_impact": categorized["MEDIUM"],
            "low_impact_count": len(categorized["LOW"]),
        },
        "recommendation": recommendation,
    }

    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
