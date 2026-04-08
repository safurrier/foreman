#!/usr/bin/env python3
"""Verify that paths referenced in AGENTS.md and docs/*.md files exist.

Checks backtick-quoted paths, routing table entries, and CLAUDE.md symlinks.
"""

import re
import subprocess
import sys
from pathlib import Path

# Patterns to skip (not file paths)
SKIP_PATTERNS = [
    r"^https?://",
    r"^git\+https?://",
    r"^git@",
    r"^\$",
    r"^[A-Z_]+=",
    r"^--",
    r"^\{",
    r"^#",
    r"^\*",
    r"^>",
    r"^collection://",
    r"^discussion://",
    r"^notion://",
    r"^~/",
    r"^~$",
    r"^/",
    r"^tap ",
    r"^brew ",
    # Homebrew tap names (e.g., homebrew/cask-fonts, knqyf263/pet)
    r"^homebrew/",
    # Keyboard shortcuts (e.g., Alt+H/L, Ctrl+Left/Right, Shift+a-z)
    r"^(Alt|Ctrl|Shift|Meta|Cmd|Super)\+",
    # Vim-style key notation (e.g., <leader>gs, <prefix>)
    r"^<(leader|prefix|space|cr|esc|tab|bs|del|up|down|left|right)[^>]*>",
    # Cloud storage / container registry URIs
    r"^gs://",
    r"^gcr\.io/",
    r"^docker\.io/",
    # Domain-like patterns (e.g., discord.dagster.cloud/, pkg.go.dev/)
    r"^\w+\.\w+\.\w+/",
    # Python decorators (e.g., @pytest.mark.e2e)
    r"^@",
    # GitHub org/repo format (e.g., safurrier/ai-config) — single slash, no file extension
    r"^[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+$",
    # Dotfile/output directory paths (e.g., .codex/, .claude/, .groundskeeper/)
    # These describe tool output in the user's home dir, not repo paths
    r"^\.[a-zA-Z][\w-]*/",
    r"^\.[a-zA-Z][\w-]*$",
]


def strip_code_fences(content: str) -> str:
    """Remove triple-backtick code blocks from markdown content."""
    return re.sub(r"```[\s\S]*?```", "", content)


def get_git_root(path: Path) -> Path | None:
    """Get the git root directory, if in a git repo."""
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        cwd=path,
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        return Path(result.stdout.strip())
    return None


def is_likely_path(text: str) -> bool:
    """Heuristic: does this backtick-quoted string look like a file path?

    Conservative: only match things that look like relative file/dir paths
    within the repo. Skip commands, absolute paths, home-relative paths,
    short names, and anything with spaces (likely a command).
    """
    text = text.strip()

    if not text or len(text) < 3:
        return False

    for pattern in SKIP_PATTERNS:
        if re.match(pattern, text):
            return False

    # Skip anything with spaces — likely a command, not a path
    if " " in text:
        return False

    # Skip things that look like code (has parens, equals, etc.)
    if any(c in text for c in ("(", ")", "=", ";", "{", "}", "<", ">", '"', "'")):
        return False

    # Skip things that contain template markers
    if "{{" in text or "<" in text:
        return False

    # Must contain a slash (relative path) or a dot-extension (filename)
    # Simple names without slashes or extensions are likely not paths
    has_slash = "/" in text
    has_extension = bool(re.search(r"\.\w{1,10}$", text))

    if not has_slash and not has_extension:
        return False

    # Bare file extensions (e.g., `.tmpl`, `.py`) — mentioning a type, not a file
    if not has_slash and text.startswith(".") and re.fullmatch(r"\.\w+", text):
        return False

    # Dot-notation config keys (e.g., `training.epochs`, `conversion.targets`)
    # No slash, dots between word chars — likely a config key, not a path
    if not has_slash and re.fullmatch(r"\w+(\.\w+)+", text):
        return False

    # Docker/container image patterns with tags (e.g., `python:3.12`, `image:latest`)
    if ":" in text and re.search(r":\w[\w.-]*$", text):
        return False

    # Skip dotted property access patterns (e.g., .parent, chezmoi.sourceDir)
    # Real filenames with a single dot have a short extension (.md, .py, .toml)
    # Property access has camelCase or long "extensions" that aren't file types
    if not has_slash and "." in text:
        _, ext = text.rsplit(".", 1)
        # Skip if "extension" contains uppercase (property access like .sourceDir, .chezmoi)
        if ext and any(c.isupper() for c in ext):
            return False
        # Skip if entire text starts with "." and "extension" is a common method name
        if text.startswith(".") and ext in ("parent", "name", "stem", "suffix", "resolve"):
            return False

    # Skip glob/wildcard patterns
    if "*" in text or "?" in text:
        return False

    return True


def extract_backtick_paths(content: str) -> list[str]:
    """Extract file paths from backtick-quoted strings in markdown."""
    paths = []

    # Single backtick paths (not code blocks)
    for match in re.finditer(r"`([^`\n]+)`", content):
        candidate = match.group(1).strip()
        if is_likely_path(candidate):
            # Strip trailing punctuation that's not part of the path
            candidate = candidate.rstrip(",;:")
            paths.append(candidate)

    return paths


def extract_table_paths(content: str) -> list[str]:
    """Extract file paths from markdown table cells."""
    paths = []

    for line in content.splitlines():
        if not line.strip().startswith("|"):
            continue

        cells = [c.strip() for c in line.split("|")[1:-1]]
        for cell in cells:
            # Look for backtick paths in cells
            for match in re.finditer(r"`([^`]+)`", cell):
                candidate = match.group(1).strip()
                if is_likely_path(candidate):
                    paths.append(candidate)

    return paths


def check_symlinks(root: Path) -> list[str]:
    """Check that CLAUDE.md files are symlinks to AGENTS.md where expected."""
    errors = []

    for agents_md in root.rglob("AGENTS.md"):
        if any(
            part.startswith(".") or part == "node_modules"
            for part in agents_md.relative_to(root).parts
        ):
            continue

        claude_md = agents_md.parent / "CLAUDE.md"
        if claude_md.is_symlink():
            # Check symlink target is valid and points to AGENTS.md
            if not claude_md.exists():
                errors.append(f"{claude_md}: broken symlink (target does not exist)")
            else:
                target = claude_md.resolve()
                if target != agents_md.resolve():
                    errors.append(f"{claude_md}: symlink points to {target}, expected {agents_md}")

    return errors


def verify_file(path: Path, root: Path, git_root: Path | None = None) -> list[str]:
    """Verify all path references in a single file."""
    errors = []
    raw_content = path.read_text(encoding="utf-8")
    file_dir = path.parent

    # Strip code fences before extracting paths to avoid false positives
    content = strip_code_fences(raw_content)

    # Collect all referenced paths
    paths = extract_backtick_paths(content)
    paths.extend(extract_table_paths(content))

    # Deduplicate
    seen = set()
    unique_paths = []
    for p in paths:
        if p not in seen:
            seen.add(p)
            unique_paths.append(p)

    for ref_path in unique_paths:
        # Strip file:line references (e.g., "path/to/file.py:58-87" or "path:42")
        clean_path = re.sub(r":\d+(-\d+)?$", "", ref_path)

        # Resolve relative to the file's directory
        candidate = file_dir / clean_path
        if not candidate.exists():
            # Also try relative to provided root
            candidate = root / clean_path
            if not candidate.exists():
                # Also try relative to git root (monorepo fallback)
                if git_root and git_root != root:
                    candidate = git_root / clean_path
                    if candidate.exists():
                        continue
                # Skip paths that look like they contain template variables
                if "{{" in ref_path or "{%" in ref_path:
                    continue
                # Skip wildcard/glob patterns
                if "*" in ref_path:
                    continue
                errors.append(f"{path}: broken reference `{ref_path}`")

    return errors


EXCLUDED_DIRS = frozenset({".git", "node_modules", "_archive", "archive", ".venv", "__pycache__"})


def extract_markdown_link_paths(content: str) -> list[str]:
    """Extract relative paths from markdown links [text](path)."""
    paths = []
    for match in re.finditer(r"\[([^\]]*)\]\(([^)]+)\)", content):
        target = match.group(2).strip()
        # Skip absolute URLs, anchors, and non-file links
        if target.startswith(("http://", "https://", "#", "mailto:")):
            continue
        # Strip anchors from file links (e.g., "file.md#section")
        target = target.split("#")[0]
        if target:
            paths.append(target)
    return paths


def check_docs_index_completeness(root: Path) -> list[str]:
    """Check that docs/AGENTS.md lists all .md files in its docs/ directory.

    When docs/AGENTS.md exists, verify it references (in backtick-quoted paths
    within tables) all .md files in that docs/ directory, excluding README.md,
    AGENTS.md itself, and _archive/.
    """
    errors = []

    for agents_md in root.rglob("AGENTS.md"):
        rel_parts = agents_md.relative_to(root).parts
        if any(part.startswith(".") or part in EXCLUDED_DIRS for part in rel_parts):
            continue

        # Only check AGENTS.md files that are directly inside a docs/ directory
        if agents_md.parent.name != "docs":
            continue

        docs_dir = agents_md.parent
        content = agents_md.read_text(encoding="utf-8")

        # Collect all backtick-quoted paths in the AGENTS.md
        referenced = set()
        for match in re.finditer(r"`([^`\n]+)`", content):
            referenced.add(match.group(1).strip())

        # Find all .md files in the docs/ directory (recursive, skip excluded)
        for md_file in docs_dir.rglob("*.md"):
            if md_file.name in ("README.md", "AGENTS.md"):
                continue
            md_rel_parts = md_file.relative_to(root).parts
            if any(part in EXCLUDED_DIRS for part in md_rel_parts):
                continue

            # Check if this file is referenced in the AGENTS.md
            # Try both relative-to-docs and relative-to-root paths
            rel_to_docs = str(md_file.relative_to(docs_dir))
            rel_to_root = str(md_file.relative_to(root))

            if rel_to_docs not in referenced and rel_to_root not in referenced:
                errors.append(
                    f"{agents_md}: docs index missing entry for `{rel_to_docs}`"
                )

    return errors


def check_docs_readme_links(root: Path) -> list[str]:
    """Check that markdown links in docs/README.md resolve to existing files."""
    errors = []

    for readme in root.rglob("README.md"):
        rel_parts = readme.relative_to(root).parts
        if any(part.startswith(".") or part in EXCLUDED_DIRS for part in rel_parts):
            continue

        # Only check README.md files that are directly inside a docs/ directory
        if readme.parent.name != "docs":
            continue

        content = readme.read_text(encoding="utf-8")
        link_paths = extract_markdown_link_paths(content)

        for link_path in link_paths:
            candidate = readme.parent / link_path
            if not candidate.exists():
                # Also try relative to root
                candidate = root / link_path
                if not candidate.exists():
                    errors.append(
                        f"{readme}: broken link `{link_path}`"
                    )

    return errors


def find_context_files(root: Path) -> list[Path]:
    """Find AGENTS.md files and docs/*.md files to check."""
    files = []

    for agents_md in root.rglob("AGENTS.md"):
        if any(
            part.startswith(".") or part in EXCLUDED_DIRS
            for part in agents_md.relative_to(root).parts
        ):
            continue
        files.append(agents_md)

    for docs_dir in root.rglob("docs"):
        if not docs_dir.is_dir():
            continue
        rel_parts = docs_dir.relative_to(root).parts
        if any(part.startswith(".") or part in EXCLUDED_DIRS for part in rel_parts):
            continue
        # Skip vendored plugin docs (e.g., config/tmux/plugins/tmux-continuum/docs/)
        if "plugins" in rel_parts:
            parent_of_plugins = docs_dir
            while parent_of_plugins.name != "plugins":
                parent_of_plugins = parent_of_plugins.parent
            # Only skip if the plugins dir is NOT the ai-config plugins dir
            # (ai-config/plugins/ contains skill AGENTS.md files we want to check)
            if "ai-config" not in str(parent_of_plugins):
                continue
        for md in docs_dir.rglob("*.md"):
            md_rel_parts = md.relative_to(root).parts
            if any(part in EXCLUDED_DIRS for part in md_rel_parts):
                continue
            files.append(md)

    return sorted(files)


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
    root = root.resolve()

    if not root.exists():
        print(f"Error: {root} does not exist")
        return 1

    git_root = get_git_root(root)

    files = find_context_files(root)
    if not files:
        print(f"No AGENTS.md or docs/*.md files found in {root}")
        return 0

    all_errors: list[str] = []

    for f in files:
        errors = verify_file(f, root, git_root=git_root)
        all_errors.extend(errors)

    # Check symlinks
    symlink_errors = check_symlinks(root)
    all_errors.extend(symlink_errors)

    # Check docs/ index completeness
    index_errors = check_docs_index_completeness(root)
    all_errors.extend(index_errors)

    # Check docs/README.md links
    readme_errors = check_docs_readme_links(root)
    all_errors.extend(readme_errors)

    if all_errors:
        print(f"FAIL: {len(all_errors)} broken reference(s) found:\n")
        for err in all_errors:
            print(f"  - {err}")
        return 1

    print(f"PASS: {len(files)} file(s) checked, all references valid")
    return 0


if __name__ == "__main__":
    sys.exit(main())
