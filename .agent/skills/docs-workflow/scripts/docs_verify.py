#!/usr/bin/env python3
"""Verify documentation structure and integrity.

Lightweight checks for docs/ directories:
- Internal relative links resolve
- Frontmatter presence on docs/*.md files
- docs/AGENTS.md index completeness
- docs/README.md link validity

Usage:
    python scripts/docs_verify.py [root_path]

Exit codes:
    0 - all checks pass
    1 - one or more checks failed
"""

import re
import sys
from pathlib import Path

SKIP_DIRS = {".git", "node_modules", "_archive", ".venv", "__pycache__", "archive"}

MARKDOWN_LINK_RE = re.compile(r"\[([^\]]*)\]\(([^)]+)\)")
BACKTICK_PATH_RE = re.compile(r"`([^`]+\.md)`")


def find_docs_dirs(root: Path) -> list[Path]:
    """Find all docs/ directories, skipping excluded and vendored dirs."""
    results = []
    for path in root.rglob("docs"):
        if not path.is_dir():
            continue
        rel_parts = path.relative_to(root).parts
        if any(part in SKIP_DIRS for part in rel_parts):
            continue
        # Skip vendored plugin docs (e.g., config/tmux/plugins/*/docs/,
        # config/ai-config/plugins/nlspec-factory/docs/).
        # Only check docs/ dirs that are NOT inside a plugins/ subtree,
        # OR that are inside the ai-config plugins skill directories
        # (those contain our own skill docs, not vendored content).
        if "plugins" in rel_parts:
            plugins_idx = rel_parts.index("plugins")
            # Allow ai-config/plugins/*/skills/*/docs/ (our skill docs)
            after_plugins = rel_parts[plugins_idx + 1 :]
            if "skills" not in after_plugins:
                continue
        results.append(path)
    return sorted(results)


def get_md_files(docs_dir: Path) -> list[Path]:
    """Get all .md files in a docs/ directory (non-recursive into _archive)."""
    files = []
    for f in docs_dir.rglob("*.md"):
        if any(part in SKIP_DIRS for part in f.relative_to(docs_dir).parts):
            continue
        files.append(f)
    return sorted(files)


def check_internal_links(docs_dir: Path, md_files: list[Path]) -> list[str]:
    """Check that all internal markdown links resolve to existing files."""
    errors = []
    for md_file in md_files:
        content = md_file.read_text(encoding="utf-8", errors="replace")
        for match in MARKDOWN_LINK_RE.finditer(content):
            link_target = match.group(2)
            # Skip external URLs
            if link_target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            # Strip anchors
            link_path = link_target.split("#")[0]
            if not link_path:
                continue
            # Resolve relative to the file's directory
            resolved = (md_file.parent / link_path).resolve()
            if not resolved.exists():
                rel = md_file.relative_to(docs_dir)
                errors.append(f"  {rel}: broken link [{match.group(1)}]({link_target})")
    return errors


def check_frontmatter(docs_dir: Path, md_files: list[Path]) -> list[str]:
    """Check that docs/*.md files (except README.md) have YAML frontmatter."""
    errors = []
    for md_file in md_files:
        if md_file.name in ("README.md", "AGENTS.md"):
            continue
        content = md_file.read_text(encoding="utf-8", errors="replace")
        if not content.startswith("---"):
            rel = md_file.relative_to(docs_dir)
            errors.append(f"  {rel}: missing frontmatter")
    return errors


def check_index_completeness(docs_dir: Path, md_files: list[Path]) -> list[str]:
    """Check that docs/AGENTS.md references all .md files in docs/."""
    agents_md = docs_dir / "AGENTS.md"
    if not agents_md.exists():
        return []

    content = agents_md.read_text(encoding="utf-8", errors="replace")

    # Collect all backtick paths from tables in AGENTS.md
    referenced = set()
    for match in BACKTICK_PATH_RE.finditer(content):
        referenced.add(match.group(1))

    # Also check markdown links
    for match in MARKDOWN_LINK_RE.finditer(content):
        link_target = match.group(2).split("#")[0]
        if link_target.endswith(".md"):
            referenced.add(link_target)

    errors = []
    for md_file in md_files:
        if md_file.name in ("README.md", "AGENTS.md"):
            continue
        rel = md_file.relative_to(docs_dir)
        rel_str = str(rel)
        # Check if the file is referenced by any of the paths
        is_referenced = any(
            rel_str == ref or rel_str.endswith(ref) or ref.endswith(rel_str)
            for ref in referenced
        )
        if not is_referenced:
            errors.append(f"  {rel}: not referenced in AGENTS.md")

    return errors


def check_readme_links(docs_dir: Path) -> list[str]:
    """Check that all markdown links in docs/README.md resolve."""
    readme = docs_dir / "README.md"
    if not readme.exists():
        return []
    return check_internal_links(docs_dir, [readme])


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()
    root = root.resolve()

    docs_dirs = find_docs_dirs(root)
    if not docs_dirs:
        print(f"No docs/ directories found under {root}")
        return 0

    all_errors: dict[str, list[str]] = {}

    for docs_dir in docs_dirs:
        rel_docs = docs_dir.relative_to(root)
        md_files = get_md_files(docs_dir)
        if not md_files:
            continue

        dir_errors = []

        link_errors = check_internal_links(docs_dir, md_files)
        if link_errors:
            dir_errors.append("Broken internal links:")
            dir_errors.extend(link_errors)

        fm_errors = check_frontmatter(docs_dir, md_files)
        if fm_errors:
            dir_errors.append("Missing frontmatter:")
            dir_errors.extend(fm_errors)

        idx_errors = check_index_completeness(docs_dir, md_files)
        if idx_errors:
            dir_errors.append("AGENTS.md index incomplete:")
            dir_errors.extend(idx_errors)

        readme_errors = check_readme_links(docs_dir)
        if readme_errors:
            dir_errors.append("README.md broken links:")
            dir_errors.extend(readme_errors)

        if dir_errors:
            all_errors[str(rel_docs)] = dir_errors

    # Also check root README.md and root AGENTS.md links
    root_files = [root / "README.md", root / "AGENTS.md"]
    root_errors: list[str] = []
    for root_file in root_files:
        if not root_file.exists():
            continue
        content = root_file.read_text(encoding="utf-8", errors="replace")
        for match in MARKDOWN_LINK_RE.finditer(content):
            link_target = match.group(2)
            if link_target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            link_path = link_target.split("#")[0]
            if not link_path:
                continue
            resolved = (root_file.parent / link_path).resolve()
            if not resolved.exists():
                root_errors.append(
                    f"  {root_file.name}: broken link [{match.group(1)}]({link_target})"
                )
    if root_errors:
        all_errors["(root)"] = root_errors

    if all_errors:
        print("FAIL")
        for docs_path, errors in all_errors.items():
            print(f"\n{docs_path}/:")
            for err in errors:
                print(err)
        return 1

    print(f"PASS ({len(docs_dirs)} docs/ directories checked, root links verified)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
