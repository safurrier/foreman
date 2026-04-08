# Generated AGENTS.md Specification

Exact structure and content guidelines for generated AGENTS.md files. All generated
files follow this template.

## Contents
- Required structure (frontmatter, WHY/WHAT/HOW, commands, invariants, gotchas, pointers)
- CLAUDE.md strategy (symlink + consolidation workflow)
- Nested AGENTS.md guidelines
- Command validation rules
- Things to absolutely avoid

## Required Structure

### 1. YAML Frontmatter

Every generated `docs/*.md` content file gets frontmatter per `frontmatter-index-spec.md`. AGENTS.md files are routing indexes and do **not** get frontmatter -- agents auto-discover them by convention.

### 2. Project Overview (WHY)

1-3 paragraphs: what it is, who uses it, what "done" means.

### 3. Repo Map (WHAT)

Bullet list or tree of the directories that matter most.
For monorepos: identify apps/services and shared packages.

Include a "Key steering files" subsection:
- `AGENTS.md` — this file
- `docs/architecture.md` (if generated)
- Other key config/doc pointers

### 4. How to Work Here (HOW)

Short workflow guidance (explore → plan → implement → validate).
Explicit validation expectations (what must pass before committing).

### 5. Common Commands (validated)

Compact list with validation markers:
- ✅ executed successfully
- ⚠️ executed but failed (include reason + fix suggestion)
- ⏸️ not executed (include why and how to run)

**Critical rule — no fast-changing specifics:**
Never include specific counts, timing, or other numbers that go stale.

| Write this | Not this |
|-----------|---------|
| `uv run pytest tests/unit/ -v` — run the unit test suite | `uv run pytest tests/unit/ -v` — runs 159 tests in 8.2s |
| `mise run test` — run all tests (includes E2E, slower) | `mise run test` — runs 342 tests across 24 modules |
| `mise run check` — fast quality gate | `mise run check` — completes in ~15s |

Counts go stale the moment someone adds a test. The command doesn't.

### 6. System Invariants

Rules that if violated break everything downstream. Separate from gotchas.

> These rules prevent catastrophic mistakes. Violating them may require
> reverting or significant cleanup.

**Format:**
```
- **{Invariant name}**: {What must always be true}. Violation causes: {consequence}.
```

**Examples:**
- **Forward-only migrations**: Database migrations never include rollback scripts. Violation causes: schema inconsistency across environments.
- **Auth on all endpoints**: Every public API endpoint declares an auth model before merge. Violation causes: unauthenticated access to protected resources.
- **CI parity**: `mise run check` locally must match what CI runs. Violation causes: green local / red CI divergence.

**Guidelines:**
- 3-7 invariants max. If more exist, they belong in `docs/architecture.md`.
- Only include invariants discoverable from code, CI, or config — do not speculate.
- Mark unverified invariants with `REVIEW:` prefix for human review.
- Higher stakes than gotchas. Wrong invariants are worse than no invariants.

### 7. Gotchas

Every gotcha follows the **DO / NOT / BECAUSE** pattern:

**Format:**
```
- **DO** {correct action}. **NOT** {common mistake}. **BECAUSE** {rationale}.
```

**Examples:**
- **DO** run `uv run pytest tests/unit/ -v` for fast validation. **NOT** `mise run test` (runs full suite including integration). **BECAUSE** integration tests require network access and are significantly slower; unit tests provide fast feedback.
- **DO** edit brewfiles in `brewfiles/` directory. **NOT** `provisioning/ansible/vars/packages.yml` for Homebrew packages. **BECAUSE** packages.yml is for apt/dnf system packages; Homebrew has its own brewfile system.
- **DO** use `uv run` for Python commands. **NOT** bare `pip` or `python`. **BECAUSE** uv manages the virtualenv and ensures reproducible dependency resolution.

**Guidelines:**
- Only high-impact, stable surprises not obvious from code
- The redirect (what TO do) is mandatory — an alternative is worth more than a warning
- Never include specific timing or counts (apply the no-fast-changing-specifics rule)

### 8. Progressive Disclosure Pointers

Two routing tables that let agents decide what to load:

```markdown
## Task-Specific Docs

### Nested module docs (auto-discovered by Claude Code)

| Path | Topic |
|------|-------|
| `module/AGENTS.md` | Brief description |

### Cross-cutting docs in `docs/`

| Doc | Topic |
|-----|-------|
| `docs/architecture.md` | System design and invariants |
| `docs/testing.md` | Test infrastructure |
```

### 9. Key References

Links to authoritative docs, config files, external resources.

---

## Docs Index Generation

Whenever the skill creates or updates files in a `docs/` directory, generate or
update two index files. These are the entry points that make the rest of the
documentation findable.

### When to Generate

- Creating a new `docs/` directory: always create both indexes
- Adding/removing/renaming a file in `docs/`: update both indexes
- Updating a docs/ file's title or description: update the AGENTS.md index entry

### `docs/README.md` -- Human Index

Ordered by learning progression (tutorial → how-to → explanation → reference).
Curated, not exhaustive. A newcomer reads top-to-bottom.

```markdown
# Project Documentation

## Start Here
- [Fresh Install Guide](fresh-install.md) -- set up from scratch
- [Getting Started](getting-started.md) -- first steps after install

## Common Tasks
- [Decision Tree](decision-tree.md) -- "I want to X → edit Y, run Z"

## Understand the System
- [Architecture](architecture.md) -- how the pieces fit together

## Reference
- [Zsh Reference](zsh-reference.md) -- aliases, shortcuts
- [Neovim Reference](nvim-reference.md) -- keybindings, plugins
```

### `docs/AGENTS.md` -- Agent Routing Index

A routing index organized by doc type with tables. As an AGENTS.md file, it does
**not** get YAML frontmatter (agents auto-discover it by convention).

```markdown
## Tutorials
| Doc | Description |
|-----|-------------|
| `fresh-install.md` | Complete fresh machine setup |

## How-to
| Doc | Description |
|-----|-------------|
| `decision-tree.md` | "I want to X → edit Y, run Z" |

## Explanation
| Doc | Description |
|-----|-------------|
| `architecture.md` | System design and component overview |

## Reference
| Doc | Description |
|-----|-------------|
| `zsh-reference.md` | Shell aliases, AI shortcuts, bookmarks |
```

### Intent-Aware Placement

Classify each docs/ file by type using the intent classification in `references/docs-structure.md`.
Place files according to the scale rules:

- **Flat** (< 8 docs): all files at `docs/` root level
- **Folders** (8-20 docs): group by doc type; create a folder as soon as any doc of that type is identified
- **Sub-indexes** (20+ docs): add per-folder README.md when a folder hits 8+ docs

Never pre-create empty folders. The structure grows from content.

### Cross-Cutting System Example

When documenting a cross-cutting system (Scale D), the `docs/AGENTS.md` includes
a Code Locations table alongside the intent-typed tables:

```markdown
## ML Inference Pipeline -- Code Locations

| Component | Path | Role |
|-----------|------|------|
| Training | `ml/training/` | Model training and evaluation |
| Serving | `services/triton/` | Triton inference server |
| Client | `api/routes/inference.py` | API endpoint calling the model |
```

---

## CLAUDE.md Strategy

Default: CLAUDE.md is a symlink to AGENTS.md.

### Consolidation Workflow

**Detection**: Check if CLAUDE.md is a symlink (`ls -la CLAUDE.md`).

**Interactive modes (quick-start, update, evaluate)**: Ask the user:
> "CLAUDE.md exists as a standalone file. Consolidate into AGENTS.md and replace
> with a symlink?"

**Auto mode**: Consolidate automatically:
1. Read both files
2. Merge: deduplicate, prefer more accurate content, reconcile contradictions
3. Write merged content to AGENTS.md
4. Remove standalone CLAUDE.md
5. Create symlink: `ln -s AGENTS.md CLAUDE.md`
6. Report what was merged

**Merge strategy:**
- Only CLAUDE.md exists → rename to AGENTS.md, create symlink
- Both exist → merge by WHY/WHAT/HOW sections, keep most complete version of each
- Preserve unique CLAUDE.md content that doesn't exist in AGENTS.md

---

## Nested AGENTS.md Guidelines

Same WHY/WHAT/HOW structure, scoped to the module.
Always create CLAUDE.md symlink alongside.
Same validation markers. No frontmatter (AGENTS.md files are routing indexes).

---

## Command Validation Rules

For each command listed:
1. Confirm it exists (package.json scripts, Makefile target, CI step)
2. If feasible, run a non-destructive validation (`--help`, list scripts/targets, dry-run)
3. Record status as ✅ / ⚠️ / ⏸️

**Test command rules (required when feasible):**
- Attempt a minimal, safe, fast test execution to prove the harness works
- Avoid anything long-running, networked, E2E, integration, or data-mutating
- Do NOT run targets containing: `e2e`, `integration`, `playwright`, `cypress`,
  `selenium`, `docker`, `compose`, `migrate`, `seed`, `reset`, `drop`
- Prefer "unit/smoke/short" targets or "collect only/dry-run" mode
- If cannot confidently keep it fast/offline, mark ⏸️ and explain

---

## Audience Filter: Agent vs Human Content

Before routing any content, apply the **actor test**: *who does this?*

**Agent-facing → AGENTS.md** (nested, load-on-demand):
- **Script purpose tables** (when/why to run each script) — agent calls these
- **File maps** for config or source directories — agent navigates these
- **Build/test/lint commands** — agent invokes these
- **Invariants, gotchas** — agent must not violate these

**Human-facing → `docs/{topic}-reference.md` or `README.md` alongside the config**:
- **Interactive keyboard shortcuts** (editor, terminal, IDE keybindings) — humans press keys; agents don't
- **Shell aliases and shorthand commands** typed at a prompt — humans type them; agents invoke full commands
- **UI or display preferences** (color themes, pager settings, prompt style) — humans configure these
- **Daily-use cheat sheets** and personal workflow references — for humans operating the tool

The test is not about content type or durability. Stable, accurate content is still human-facing if a human is the actor. Route it to `docs/` where both humans and agents can find it on demand, but keep AGENTS.md focused on what agents actually need to do their job.

## Things to Absolutely Avoid

- Long style guides (prefer "run formatter/linter X")
- Huge auto-generated command outputs, tool listings with output (not reference tables)
- Copy-pasted code that will go stale (prefer `file:line` references)
- Fast-changing specifics (test counts, timing data, line counts, module counts)
- Speculative invariants (only document what's discoverable from code/CI)
- Human-facing content in AGENTS.md (interactive shortcuts, shell aliases, display preferences) — route to `docs/` instead
