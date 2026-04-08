#!/usr/bin/env python3
"""Validate YAML frontmatter on AGENTS.md and docs/*.md files.

Checks that generated context files have proper frontmatter with required
fields (id, title, description, index[]) and that index entries correspond
to actual headings in the document.
"""

import re
import sys
from pathlib import Path


def parse_frontmatter(content: str) -> tuple[dict | None, str]:
    """Extract YAML frontmatter from markdown content.

    Returns (frontmatter_dict, body) or (None, content) if no frontmatter.
    """
    if not content.startswith("---"):
        return None, content

    end = content.find("---", 3)
    if end == -1:
        return None, content

    raw = content[3:end].strip()
    body = content[end + 3 :].strip()

    fm: dict = {}
    current_key = None
    current_list: list | None = None

    for line in raw.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue

        # Top-level key: value
        if re.match(r"^[a-z_]+:", line) and not line.startswith(" "):
            if current_key and current_list is not None:
                fm[current_key] = current_list

            key, _, value = line.partition(":")
            key = key.strip()
            value = value.strip()

            if value == "" or value == ">":
                current_key = key
                current_list = None
                if value == ">":
                    # Folded scalar — collect continuation lines
                    fm[key] = ""
                continue

            fm[key] = value
            current_key = key
            current_list = None

        # List item under a key
        elif stripped.startswith("- ") and current_key:
            item_val = stripped[2:].strip()
            # Check if it's a list of dicts (index entries)
            if item_val.startswith("id:"):
                if current_list is None:
                    current_list = []
                entry: dict = {}
                entry_key, _, entry_val = item_val.partition(":")
                entry[entry_key.strip()] = entry_val.strip()
                current_list.append(entry)
            elif current_list is not None and isinstance(current_list, list):
                current_list.append(item_val)
            else:
                if current_list is None:
                    current_list = []
                current_list.append(item_val)

        # Continuation of a dict entry (e.g., keywords: [...])
        elif current_list and isinstance(current_list, list) and current_list:
            last = current_list[-1]
            if isinstance(last, dict):
                if ":" in stripped:
                    k, _, v = stripped.partition(":")
                    k = k.strip()
                    v = v.strip()
                    # Parse inline list [a, b, c]
                    if v.startswith("[") and v.endswith("]"):
                        items = [x.strip().strip("'\"") for x in v[1:-1].split(",") if x.strip()]
                        last[k] = items
                    else:
                        last[k] = v

        # Folded scalar continuation
        elif current_key and current_key in fm and isinstance(fm[current_key], str):
            if fm[current_key]:
                fm[current_key] += " " + stripped
            else:
                fm[current_key] = stripped

    if current_key and current_list is not None:
        fm[current_key] = current_list

    return fm, body


def extract_h2_headings(body: str) -> list[str]:
    """Extract all ## heading IDs (kebab-cased) from markdown body."""
    headings = []
    for line in body.splitlines():
        match = re.match(r"^##\s+(.+?)(?:\s*\{.*\})?\s*$", line)
        if match:
            raw = match.group(1).strip()
            # Convert to kebab-case id
            slug = re.sub(r"[^\w\s-]", "", raw.lower())
            slug = re.sub(r"[\s_]+", "-", slug).strip("-")
            headings.append(slug)
    return headings


def validate_file(path: Path, seen_ids: dict[str, Path]) -> list[str]:
    """Validate a single file's frontmatter. Returns list of error messages."""
    errors = []
    content = path.read_text(encoding="utf-8")

    fm, body = parse_frontmatter(content)
    if fm is None:
        errors.append(f"{path}: missing YAML frontmatter")
        return errors

    # Required fields
    for field in ("id", "title", "description"):
        if field not in fm:
            errors.append(f"{path}: missing required field '{field}'")

    if "index" not in fm:
        errors.append(f"{path}: missing 'index' array")
        return errors

    index = fm.get("index", [])
    if not isinstance(index, list):
        errors.append(f"{path}: 'index' must be an array")
        return errors

    # Check id uniqueness
    file_id = fm.get("id", "")
    if file_id:
        if file_id in seen_ids:
            errors.append(f"{path}: duplicate id '{file_id}' (also in {seen_ids[file_id]})")
        else:
            seen_ids[file_id] = path

    # Validate index entries
    headings = extract_h2_headings(body)

    for i, entry in enumerate(index):
        if not isinstance(entry, dict):
            errors.append(f"{path}: index[{i}] is not a mapping")
            continue

        if "id" not in entry:
            errors.append(f"{path}: index[{i}] missing 'id'")

        # Check that index id corresponds to a heading
        entry_id = entry.get("id", "")
        if entry_id and headings and entry_id not in headings:
            errors.append(
                f"{path}: index[{i}].id '{entry_id}' has no matching ## heading "
                f"(found: {', '.join(headings)})"
            )

    return errors


EXCLUDED_DIRS = frozenset({".git", "node_modules", "_archive", "archive", ".venv", "__pycache__"})


def find_context_files(root: Path) -> list[Path]:
    """Find docs/*.md content files to validate.

    AGENTS.md files are routing indexes and do not require frontmatter,
    so they are excluded from validation.
    """
    files = []

    # Find docs/**/*.md files (recursively from any docs/ directory)
    for docs_dir in root.rglob("docs"):
        if not docs_dir.is_dir():
            continue
        rel_parts = docs_dir.relative_to(root).parts
        if any(part.startswith(".") or part in EXCLUDED_DIRS for part in rel_parts):
            continue
        # Skip vendored plugin docs (e.g., config/tmux/plugins/tmux-continuum/docs/,
        # config/ai-config/plugins/nlspec-factory/docs/)
        if "plugins" in rel_parts:
            continue
        for md in docs_dir.rglob("*.md"):
            # Skip AGENTS.md — routing indexes don't need frontmatter
            if md.name == "AGENTS.md":
                continue
            # Skip files inside _archive subdirectories
            md_rel = md.relative_to(docs_dir)
            if any(part in EXCLUDED_DIRS for part in md_rel.parts):
                continue
            files.append(md)

    return sorted(files)


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
    root = root.resolve()

    if not root.exists():
        print(f"Error: {root} does not exist")
        return 1

    files = find_context_files(root)
    if not files:
        print(f"No AGENTS.md or docs/*.md files found in {root}")
        return 0

    seen_ids: dict[str, Path] = {}
    all_errors: list[str] = []

    for f in files:
        errors = validate_file(f, seen_ids)
        all_errors.extend(errors)

    if all_errors:
        print(f"FAIL: {len(all_errors)} error(s) found:\n")
        for err in all_errors:
            print(f"  - {err}")
        return 1

    print(f"PASS: {len(files)} file(s) validated, all frontmatter correct")
    return 0


if __name__ == "__main__":
    sys.exit(main())
