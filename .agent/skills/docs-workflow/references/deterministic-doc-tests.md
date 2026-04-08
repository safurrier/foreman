# Deterministic Documentation Tests

Automated tests that enforce documentation conventions. These patterns
can be adopted by any repo using pytest.

## What to Test

Six categories of machine-checkable documentation quality:

### 1. Folder Structure

Every doc lives in an intent-based folder (`tutorials/`, `how-to/`,
`explanation/`, `reference/`). No loose `.md` files at the `docs/` root.
This enforces intent-based classification at the filesystem level.

### 2. Frontmatter Presence + Required Fields

Every doc has YAML frontmatter with `id`, `title`, `description`.
Frontmatter enables machine-readable routing and progressive disclosure
without parsing the full document.

### 3. Internal Link Validity

All markdown links (`[text](path)`) resolve to existing files. Skip
external URLs, mailto links, and anchor-only references. Resolve paths
relative to the file containing the link.

### 4. Index Completeness

`docs/AGENTS.md` references every doc in `docs/` (excluding AGENTS.md,
README.md, and `_archive/`). Match by filename or relative path from
backtick-quoted paths and markdown link targets.

### 5. Frontmatter Index Matches Headings

`index[]` entries in frontmatter correspond to actual `## ` headings in
the document body. Convert headings to kebab-case slugs and verify each
`- id:` value has a matching heading.

### 6. No Broken Archive References

Active docs (outside `_archive/`) must not contain markdown links
pointing into `_archive/`. If content was archived, references to it
should be updated or removed.

## Example Implementation

```python
"""Deterministic doc convention tests — pytest patterns."""

import re
from pathlib import Path

import pytest

DOCS_ROOT = Path("docs")
EXCLUDED_FILENAMES = {"AGENTS.md", "CLAUDE.md", "README.md"}
REQUIRED_FRONTMATTER_FIELDS = {"id", "title", "description"}
INTENT_FOLDERS = {"tutorials", "how-to", "explanation", "reference"}
MARKDOWN_LINK_RE = re.compile(r"\[([^\]]*)\]\(([^)]+)\)")


def iter_docs_markdown() -> list[Path]:
    """All markdown files under docs/, excluding archives and index files."""
    return [
        p for p in DOCS_ROOT.rglob("*.md")
        if p.name not in EXCLUDED_FILENAMES and "_archive" not in p.parts
    ]


def extract_markdown_links(content: str) -> list[tuple[str, str]]:
    """Return (text, target) pairs for all markdown links."""
    return MARKDOWN_LINK_RE.findall(content)


def slugify_heading(heading: str) -> str:
    """Convert heading text to kebab-case slug."""
    slug = heading.strip().lower()
    slug = re.sub(r"[^a-z0-9\s-]", "", slug)
    slug = re.sub(r"[\s]+", "-", slug)
    return re.sub(r"-+", "-", slug).strip("-")


# -- Folder structure --
@pytest.mark.docs
@pytest.mark.parametrize("doc_path", iter_docs_markdown())
def test_docs_in_intent_folders(doc_path: Path) -> None:
    relative = doc_path.relative_to(DOCS_ROOT)
    assert len(relative.parts) >= 2, f"Loose file at docs/ root: {doc_path}"
    assert relative.parts[0] in INTENT_FOLDERS


# -- Frontmatter --
@pytest.mark.docs
@pytest.mark.parametrize("doc_path", iter_docs_markdown())
def test_docs_have_frontmatter(doc_path: Path) -> None:
    content = doc_path.read_text(encoding="utf-8")
    assert content.startswith("---\n"), f"Missing frontmatter: {doc_path}"


# -- Internal links --
@pytest.mark.docs
@pytest.mark.parametrize("doc_path", iter_docs_markdown())
def test_docs_internal_links_resolve(doc_path: Path) -> None:
    content = doc_path.read_text(encoding="utf-8")
    for text, target in extract_markdown_links(content):
        if target.startswith(("http://", "https://", "mailto:", "#")):
            continue
        path_part = target.split("#")[0]
        if not path_part:
            continue
        resolved = (doc_path.parent / path_part).resolve()
        assert resolved.exists(), f"Broken link [{text}]({target}) in {doc_path}"


# -- Index completeness --
@pytest.mark.docs
def test_docs_agents_index_completeness() -> None:
    agents_path = DOCS_ROOT / "AGENTS.md"
    content = agents_path.read_text(encoding="utf-8")
    backtick_paths = re.findall(r"`([^`]*\.md)`", content)
    link_targets = [t for _, t in extract_markdown_links(content) if t.endswith(".md")]
    ref_filenames = {Path(r).name for r in backtick_paths + link_targets}
    for doc in iter_docs_markdown():
        assert doc.name in ref_filenames, f"{doc} not referenced in AGENTS.md"
```

## Configuration

- **pytest marker**: `docs` (register in `pyproject.toml` markers list)
- **Excluded filenames**: AGENTS.md, README.md, CLAUDE.md
- **Excluded directories**: `_archive/`, `node_modules/`, `.git/`
- **Valid intent folders**: `tutorials/`, `how-to/`, `explanation/`, `reference/`

## Philosophy

Deterministic verification over subjective quality. If it can be
machine-checked, check it mechanically. Fix structural issues first,
editorial quality second.

These tests catch:
- Docs added without frontmatter (CI fails immediately)
- Links that rot when files move or get archived
- Docs that slip through without being indexed
- Content that breaks the intent-folder taxonomy

They do not catch:
- Poor writing quality (subjective)
- Incomplete content (requires domain knowledge)
- Outdated instructions (requires execution)
