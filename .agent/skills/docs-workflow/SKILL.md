---
name: docs-workflow
description: Comprehensive documentation workflow for creating, generating, and maintaining technical documentation. This skill activates when users want to analyze documentation needs, generate module quickstart guides, sync docs with code changes, or consolidate scattered docs. Supports four modes - analyze (plan docs), quickstart (generate onboarding docs), update (sync with code changes), and consolidate (inventory and tidy existing docs).
---

# Documentation Workflow

A complete documentation lifecycle skill covering analysis, generation, and maintenance.

## Activation

This skill activates when users:
- Ask to "document this module", "create docs", "write documentation"
- Request "quickstart guide", "onboarding docs", "module docs"
- Want to "update docs", "sync documentation", "check if docs are current"
- Say "analyze documentation", "audit docs", "plan documentation"

## Four Modes

| Mode | Purpose | When to Use |
|------|---------|-------------|
| **analyze** | Plan documentation strategy | Starting fresh, auditing existing docs |
| **quickstart** | Generate module onboarding docs | New module needs docs, comprehensive coverage |
| **update** | Sync docs with code changes | After code changes, maintenance |
| **consolidate** | Inventory and tidy existing docs | Scattered, stale, or misplaced docs across the repo |

All modes end with a deterministic verification step. See the Verification section below.

## Mode Selection

If the user doesn't specify a mode, infer from context:
- Existing docs + recent code changes → **update**
- No docs exist for the target → **quickstart**
- User wants to plan or assess → **analyze**
- Docs have accumulated and need triage → **consolidate**
- Unclear → ask which mode

**Intent-based classification**: When selecting a mode or classifying content, use the intent classification to identify what type of documentation is needed (tutorial, how-to, explanation, reference). See `references/docs-structure.md` for the classification and `references/principles.md` P1 for the intent-first taxonomy principle.

---

## Verification

All modes end with deterministic verification. The verification script is bundled with this skill at `scripts/docs_verify.py` (relative to the skill directory, NOT the target repo). Run it by reading the script content and executing it against the target repo root:

```bash
python <skill-dir>/scripts/docs_verify.py <target-repo-root>
```

If the script is not directly executable (e.g., the agent cannot resolve the skill directory), replicate its checks manually:
- Internal links resolve to existing files
- Frontmatter is present on docs/*.md content files (excluding README.md and AGENTS.md)
- `docs/AGENTS.md` index is complete (references all docs/ files)
- `docs/README.md` links are valid

This implements `references/principles.md` P3 (deterministic verification over subjective quality).

See `references/deterministic-doc-tests.md` for pytest-based test patterns that repos can adopt.

---

## Mode: Analyze

**Purpose**: Audit existing documentation and create a documentation plan.

**Use when**: Starting fresh, assessing documentation needs, planning what to write.

**Process**:
1. Study any reference documentation as exemplars
2. Scan existing docs (e.g., `/docs` directory)
3. Identify gaps, outdated content, or temporary documentation
4. Create a documentation plan

**Output**: For each proposed document:
- Document title and output location
- Table of contents / structured outline
- 100-200 word writing sample demonstrating style

**Validation checklist** (verify before finishing):
- [ ] Every proposed doc has title, location, TOC, writing sample, priority, and effort estimate
- [ ] Commands in writing samples are annotated as verified (confirmed from source) or unverified
- [ ] No speculative content; UNKNOWN used where details cannot be confirmed from code
- [ ] Output follows the structured plan format from `references/analyze.md`

**Detailed instructions**: Load `references/analyze.md`

---

## Mode: Quickstart

**Purpose**: Generate one-screen, high-signal onboarding docs for a module.

**Use when**: A module needs documentation from scratch or comprehensive refresh.

**Inputs** (gather from user or infer):
- `feature_dir`: Repo-relative path to the module
- `tech_stack`: Languages, frameworks, services
- `scope`: What modules are covered
- `known_docs`: Any existing related docs

**Output location**: `<feature_dir>/docs/`

**Generated files** (reading order set by `tour.md` reading guide):
```
tour.md           # Quickstart + module overview (≤500 words + 1 diagram)
architecture.md   # Structure with diagrams (≤5 diagrams total)
interfaces.md     # Public API, events, types
data.md           # Tables, cache keys, invariants (only if applicable)
exercises.md      # Up to 5 learning tasks
tests.md          # Only if tests exist
```

**Diagram rules**:
- Prefer Mermaid `flowchart LR`
- ≤10 nodes per diagram, ≤5 diagrams total
- No parentheses `()` in node text
- Add `Updated: YYYY-MM-DD` below each

**Detailed instructions**: Load `references/quickstart.md`

---

## Mode: Update

**Purpose**: Sync documentation with code changes.

**Use when**: Code has changed and docs may be stale.

**Auto-detection** (in order):
1. Uncommitted changes in working directory
2. Current branch vs main/master
3. Last 10 commits on current branch
4. Last 7 days of commits (fallback)

**Change categories**:
- 🔴 **Breaking**: API changes, removed features, changed behaviors
- 🟡 **Feature**: New functionality, endpoints, options
- 🟢 **Enhancement**: Performance, UI/UX improvements
- **Fix**: Bug fixes affecting documented behavior

**Sub-modes**:
- `PLAN` (default): Present update report, ask which to proceed with
- `AUTO`: Execute critical and important updates automatically

**Output format**:
```markdown
## Documentation Update Report

### Summary
- X files need updates
- Y new documents recommended
- Z deprecated sections to remove

### Priority Actions
#### 🔴 Critical Updates
#### 🟡 Important Updates
#### 🟢 Minor Updates

### Detailed Changes
[File-by-file instructions]
```

**Detailed instructions**: Load `references/update.md`

---

## Mode: Consolidate

**Purpose**: Inventory all docs in a repo, classify each file, and produce a manifest to clean up stale, scattered, or misplaced content.

**Use when**: Docs have accumulated over time and need triage — finished migration plans, redundant references, human-facing content in AGENTS.md, or gaps with no doc.

**Sub-modes**:
- `PLAN` (default): Produce inventory report and manifest; present before touching any files
- `AUTO`: Execute manifest after showing it

**Classification**: First classify each file by doc type (tutorial/how-to/explanation/reference) and audience (agent/human). Then derive the action:
- **Keep**: Accurate reference docs, architecture docs, how-to guides still actively used
- **Archive** → `docs/_archive/`: Completed migration plans, one-time runbooks for finished work, superseded content
- **Move**: Human-facing content in AGENTS.md → `docs/{topic}-reference.md`; misplaced docs
- **Create**: Identified gaps (e.g., no routing index, missing reference doc for a configured tool)
- **Rename**: Naming inconsistencies vs. derived conventions

**Output format**:
```markdown
## Consolidation Report

### Inventory
- X files in docs/
- Y nested AGENTS.md files reviewed for misplaced content
- Z root .md files

### Manifest
#### Keep
- docs/architecture.md — accurate architecture reference
#### Archive → docs/_archive/
- docs/migration-plan-2024.md — migration complete, superseded
#### Move
- config/editor/AGENTS.md (keybindings section) → docs/editor-reference.md
#### Create
- docs/AGENTS.md — routing index for docs/

### Actions pending: N
```

**Detailed instructions**: Load `references/consolidate.md`

---

## Universal Guidelines

See `references/principles.md` for the full set of documentation principles that guide all modes.

**Writing style**:
- Concise, factual, imperative voice
- Prefer examples over prose
- Every documented element should serve a purpose
- Good docs are discoverable, actionable, maintenance-friendly

**Frontmatter**: All generated `docs/*.md` content files get simplified YAML frontmatter. AGENTS.md files are routing indexes and do not require frontmatter -- agents auto-discover them by convention.
```yaml
---
id: kebab-case-id
title: Human Title
description: >
  1-2 sentences for routing.
index:
  - id: section-slug
---
```

**Indexes**: All modes that create or move docs should create or update two index files:
- `docs/README.md` -- human progression (tutorial -> how-to -> explanation -> reference)
- `docs/AGENTS.md` -- agent routing by doc type with frontmatter

See `references/docs-structure.md` for index format and scaling rules.

**Guardrails**:
- Don't speculate; use UNKNOWN where details are missing
- Ask before running long-lived or side-effecting processes
- Verify commands work before documenting them
- Keep each page to two screens or fewer; link deeper where necessary

**Scope boundary**: This skill owns docs/ structure, indexes, and content file management.
AGENTS.md files (root and nested) are owned by the context-engineering skill. If both
skills are being used, this skill handles docs/ operations while context-engineering
handles AGENTS.md routing tables.

**Final editorial pass** (always do):
- Remove unverified or speculative details
- Dedupe redundant content across files
- Remove verbose explanations that don't change decisions or actions
