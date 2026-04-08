# docs/ Directory Guide

`docs/` is the overflow directory for content that doesn't fit in a lean AGENTS.md.
AGENTS.md stays as a routing index; `docs/` provides depth on demand. Every file in
`docs/` gets frontmatter so agents can route to it without reading the full content.

## Directory Structure

The structure scales progressively based on intent-based classification. See
`references/docs-structure.md` for the full spec.

### Intent Folders (< 8 docs)

Create an intent folder as soon as any doc of that type is identified. Only create
a folder when there's at least one doc for it (no empty folders).

```
docs/
  README.md              # Human index (learning progression)
  AGENTS.md              # Agent routing index (frontmatter)
  tutorials/
    fresh-install.md
  how-to/
    decision-tree.md
  explanation/
    architecture.md
  reference/
    zsh-reference.md
  _archive/              # Retired docs
```

### Growing Folders (8-20 docs)

As content accumulates, folders grow naturally:

```
docs/
  README.md
  AGENTS.md
  tutorials/
    fresh-install.md
    getting-started.md
    first-plugin.md
  how-to/
    decision-tree.md
    add-new-tool.md
  explanation/
    architecture.md
  reference/
    zsh-reference.md
    nvim-reference.md
  _archive/
```

Rules:
- `reference/` can have subfolders (e.g., `tools/`) when tool-specific refs accumulate
- No per-folder index files at this scale

### Sub-Indexes (20+ docs)

Only at this scale do per-folder indexes become useful. A folder gets its own
`README.md` when it hits **8+ docs**.

### The Progression Rule

```
Create an intent folder as soon as any doc of that type is identified → sub-indexes when a folder hits 8+
```

Only create a folder when there's at least one doc for it (no empty folders).

## Index Files

Every `docs/` directory gets two index files:

### `docs/README.md` -- Human Index

Ordered by learning progression (tutorial -> how-to -> explanation -> reference).
Curated, not exhaustive. A newcomer reads top-to-bottom. Uses markdown links.

### `docs/AGENTS.md` -- Agent Routing Index

Frontmatter with `id`/`title`/`description`/`index`. Body organized by doc
type with tables of backtick-quoted paths and descriptions.

See `references/docs-structure.md` for concrete examples of both index files.

## When the Repo Already Has docs/

Many repos already have a `docs/` directory with human-focused content (API docs,
MkDocs sites, README supplements). In this case:

- **Colocate**: Add agent context files alongside existing docs
- **Frontmatter distinguishes**: Existing docs won't have the index frontmatter;
  agent-generated docs will. The frontmatter is the signal.
- **Don't reorganize existing docs**: Add new files, don't move existing ones

## Cross-Cutting System Docs

When a system spans multiple code paths (e.g., an ML pipeline across `ml/training/`,
`services/triton/`, `api/routes/inference.py`), docs live **alongside the primary
module** -- the most central component. The user indicates which module is primary,
or the skill infers from which path has the most code / is most upstream.

The system doc's AGENTS.md includes a Code Locations table listing all participating
paths. See `references/docs-structure.md` Scale D for details.

## Content Routing

| Content type | Audience | Destination |
|---|---|---|
| System design, data flows, architecture | Both | `docs/{topic}.md` |
| File maps, module commands, plugin lists | Agent | `{module}/AGENTS.md` |
| Script purposes and when-to-use | Agent | `scripts/AGENTS.md` |
| Interactive keyboard shortcuts (editor, terminal, IDE) | Human | `docs/{tool}-reference.md` |
| Shell aliases and shorthand commands typed at a prompt | Human | `docs/{tool}-reference.md` |
| UI/display preferences (themes, pager settings, prompt style) | Human | `docs/{tool}-reference.md` |
| Fast-changing specifics (counts, timing) | -- | Discard |
| Unverifiable content | -- | Keep at source path, report |

## Sizing

- Each file: target under 100 lines, hard stop at ~150
- If a file would exceed 150 lines, split it
- Docs indexes: curated, not exhaustive
- If `docs/` would exceed ~15 files at flat scale, apply intent folders

## Every File Gets Frontmatter

See `frontmatter-index-spec.md`. This is what makes `docs/` work -- agents read
frontmatter to decide what to load, not the filename.

## Referencing from AGENTS.md

AGENTS.md must include a routing table listing docs/ files:

```markdown
## Task-Specific Docs

### Nested module docs (auto-discovered by Claude Code)

| Path | Topic |
|------|-------|
| `config/nvim/AGENTS.md` | Neovim configuration |
| `services/api/AGENTS.md` | API service |

### Cross-cutting docs in `docs/`

| Doc | Topic |
|-----|-------|
| `docs/architecture.md` | System design, invariants, truth hierarchy |
| `docs/testing.md` | Test infrastructure and CI integration |
| `docs/conventions.md` | Code patterns and naming |
```

## Decision Guidance

| Signal | Action |
|--------|--------|
| Module has distinct workflows/commands | Create nested `AGENTS.md` |
| Topic spans modules or is architectural | Create `docs/{topic}.md` |
| Multiple non-nested code paths for one system | Create cross-cutting system doc (Scale D) |
| Unsure | Prefer colocated nested AGENTS.md |
| Content is human-focused (API docs, tutorials) | Leave in existing `docs/`, don't add frontmatter |

Every nested AGENTS.md gets a CLAUDE.md symlink alongside it.

## Content Guidelines

- Self-contained: a reader shouldn't need to load another doc to understand this one
- Use `file:line` references instead of copying code
- Same validation markers (✅/⚠️/⏸️) for any commands
- Include cross-references to related docs at the bottom
- No fast-changing specifics (test counts, timing data, etc.)

## Validation

Run `scripts/verify_references.py` to confirm all docs/ files referenced in
AGENTS.md routing tables actually exist. The script also checks that `docs/AGENTS.md`
lists all `.md` files in its directory, and that `docs/README.md` links resolve.
