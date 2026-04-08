# Documentation Structure by Intent

How to classify documentation by intent and organize it into a scalable directory
structure. Four doc types (tutorial, how-to, explanation, reference) adapted for
AI-generated documentation that must work at multiple scales.

## Intent Classification

Two questions classify any piece of documentation:

1. **Action or cognition?** — Does this help the reader *do something* or *understand something*?
2. **Acquisition or application?** — Is the reader *learning* or *working*?

| | Acquisition (learning) | Application (working) |
|---|---|---|
| **Action** (doing) | Tutorial | How-to |
| **Cognition** (thinking) | Explanation | Reference |

Use these questions flexibly:
- *Am I writing for doing or thinking?*
- *Is the reader studying or working?*
- *Does this content serve learning or immediate use?*

Apply at the level of individual docs, not individual sentences. A doc that mixes
types should be split.

### Tutorial
*Learning-oriented, practical steps.*

The reader follows along to acquire a skill. Ordered, progressive, guided.
- Getting started guides
- Fresh install walkthroughs
- "Build your first X" docs

### How-to
*Task-oriented, practical steps.*

The reader has a specific goal and needs to accomplish it. Problem → solution.
- "I want to add a new tool" → decision tree
- Deployment runbooks
- Migration guides

### Explanation
*Understanding-oriented, theoretical.*

The reader wants to know *why* something works the way it does. Architecture,
tradeoffs, design decisions.
- System architecture docs
- Design decision records
- "Why we chose X over Y"

### Reference
*Information-oriented, precise lookup.*

The reader needs a specific fact. Tables, keybindings, CLI flags, API specs.
- Keyboard shortcut references
- Tool configuration reference
- CLI command reference

---

## Adapted Directory Structure

The structure scales progressively. Don't impose folders on a repo with 4 docs.

### Scale 1: Intent Folders (< 8 docs total)

Create an intent folder as soon as any doc of that type is identified. Only create
a folder when there's at least one doc for it (no empty folders).

```
docs/
  README.md              # Human index (learning progression)
  AGENTS.md              # Agent routing index (no frontmatter)
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

Frontmatter on each content file carries the doc type signal. AGENTS.md files do not get frontmatter.

### Scale 2: Growing Folders (8-20 docs)

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
    debug-ci-failures.md
  explanation/
    architecture.md
    provisioning-design.md
  reference/
    zsh-reference.md
    nvim-reference.md
    tools/
      lazygit.md
      eza.md
  _archive/
```

Rules:
- `reference/` can have subfolders (e.g., `tools/`) when tool-specific refs accumulate
- No per-folder index files — the top-level indexes are the only entry points

### Scale 3: Sub-Indexes (20+ docs)

Only at this scale do per-folder indexes become useful:

```
docs/
  README.md
  AGENTS.md
  tutorials/
    README.md            # Folder-level human index
    ...
  reference/
    README.md
    tools/
      README.md
      ...
```

Threshold: a folder hits **8+ docs** → add a folder-level README.md.

### The Progression Rule

```
Create an intent folder as soon as any doc of that type is identified → sub-indexes when a folder hits 8+
```

Only create a folder when there's at least one doc for it (no empty folders).

This is the **default for new documentation** (quick-start mode, quickstart mode).
When working with existing docs (update/consolidate modes), derive conventions
from what exists first. Don't reorganize a working structure to match the template
unless explicitly requested or the current structure is causing problems.

Framework-managed docs/ directories are never reorganized — see above.

---

## Two Index Files

Every `docs/` directory (at any scope) gets two indexes:

### `docs/README.md` — Human Index

Ordered by **learning progression** (tutorial → how-to → explanation → reference):

```markdown
# Project Documentation

## Start Here
- [Fresh Install Guide](tutorials/fresh-install.md) — set up from scratch
- [Getting Started](tutorials/getting-started.md) — first steps after install

## Common Tasks
- [Decision Tree](how-to/decision-tree.md) — "I want to X → edit Y, run Z"

## Understand the System
- [Architecture](explanation/architecture.md) — how the pieces fit together

## Reference
- [Zsh Reference](reference/zsh-reference.md) — aliases, shortcuts
- [Neovim Reference](reference/nvim-reference.md) — keybindings, plugins
```

Curated, not exhaustive. A newcomer reads top-to-bottom.

### `docs/AGENTS.md` — Agent Routing Index

A routing index organized by doc type. As an AGENTS.md file, it does **not** get
YAML frontmatter — agents auto-discover it by convention.

Body has per-type tables:

```markdown
## Tutorials
| Doc | Description |
|-----|-------------|
| `tutorials/fresh-install.md` | Complete fresh machine setup |

## How-to
| Doc | Description |
|-----|-------------|
| `how-to/decision-tree.md` | "I want to X → edit Y, run Z" |

## Explanation
| Doc | Description |
|-----|-------------|
| `explanation/architecture.md` | System design and component overview |

## Reference
| Doc | Description |
|-----|-------------|
| `reference/zsh-reference.md` | Shell aliases, AI shortcuts, bookmarks |
```

---

## Framework-Managed docs/

When `docs/` is managed by a documentation framework (MkDocs, Docusaurus, Sphinx,
mdBook, etc.), the framework owns the structure. Detection signals: `mkdocs.yml`,
`docusaurus.config.js`, `conf.py`, `book.toml`, or similar build configs near the
docs/ directory.

When detected, adjust behavior at agent discretion:
- **Don't reorganize into intent folders** — the framework controls file layout
- **Don't create `docs/README.md`** — the framework has its own index (e.g., `docs/index.md`)
- **Do create `docs/AGENTS.md`** — agent routing index, doesn't conflict with frameworks
- **Do add frontmatter to content files** — frameworks ignore unknown frontmatter fields
- **Note in report**: file moves require updating the framework's nav config

If the user's AGENTS.md or CLAUDE.md explicitly says to retain the existing docs
structure, respect that override regardless of whether a framework is detected.

---

## Scope Scaling

The skill works at four scales. The principles and frontmatter spec are identical
across all scales — only the volume and structure of output change.

### Scale A: Whole Small Repo

Scope: `.` or omitted. One root AGENTS.md, one `docs/` directory.

### Scale B: Small Module

Scope: `services/payments/`. Module-scoped AGENTS.md + flat `docs/` within module.
Parent AGENTS.md gets a routing table entry.

### Scale C: Large Module

Scope: `services/payments/`. Module-scoped AGENTS.md + intent-folder `docs/`.
Parent AGENTS.md gets a routing table entry.

### Scale D: Cross-Cutting System

Scope: user provides a system name + multiple code paths (e.g., "ML Inference Pipeline"
spanning `ml/training/`, `services/triton/`, `api/routes/inference.py`).

Docs live **alongside the primary module** (the most central component). The user
indicates which module is primary, or the skill infers from which path has the most
code / is most upstream.

```
services/triton/              # Primary module (serving = most central)
  AGENTS.md                   # Module AGENTS.md, includes system overview
  docs/
    README.md
    AGENTS.md
    ml-inference-pipeline.md  # Cross-cutting system doc
```

The system doc's AGENTS.md section lists all code locations:

```markdown
## ML Inference Pipeline — Code Locations

| Component | Path | Role |
|-----------|------|------|
| Training | `ml/training/` | Model training and evaluation |
| Serving | `services/triton/` | Triton inference server |
| Client | `api/routes/inference.py` | API endpoint calling the model |
| Features | `data/features/` | Feature store and preprocessing |
```

This keeps docs colocated (principle: locality) while still documenting the full
cross-cutting flow.

---

## Detection: When to Use Which Scale

| Signal | Scale |
|--------|-------|
| Single directory provided, < 8 docs | A or B (flat) |
| Single directory provided, 8+ docs | C (intent folders) |
| Multiple non-nested paths provided | D (cross-cutting system) |
| No scope provided, small repo | A (whole repo, flat) |
| No scope provided, monorepo | Infer from git activity, ask if ambiguous |

The skill detects scale by counting existing docs and examining the scope input.
It never pre-creates structure that the content doesn't warrant.
