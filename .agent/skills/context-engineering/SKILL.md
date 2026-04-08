---
name: ai-context-engineering-files
description: >
  Creates, updates, or evaluates AGENTS.md and CLAUDE.md files for a repository.
  This skill activates when users want to create onboarding docs, update agent
  context files, or evaluate documentation quality. Generates docs with YAML
  frontmatter indexes for machine-readable progressive disclosure, system
  invariants, redirect-pattern gotchas (DO/NOT/BECAUSE), and validated commands.
  Supports incremental updates via git-based staleness detection.
---

Produce and maintain **high-signal agent onboarding memory files** —
`AGENTS.md` (cross-tool, open format) and `CLAUDE.md` (Claude Code context file,
symlinked to AGENTS.md).

## Core Principles

* **Stateless onboarding**: zero prior knowledge assumed; files onboard future sessions.
* **Less is more**: root files under 150 lines; hard stop at ~300.
* **WHY / WHAT / HOW**: every file follows this structure.
* **Frontmatter-first**: YAML frontmatter with `id`, `title`, `description`, `index[]` on content docs. AGENTS.md routing indexes are exempt. See `resources/frontmatter-index-spec.md`.
* **No fast-changing specifics**: no test counts, timing data, file counts, or numbers that go stale.
* **Audience filter**: apply the actor test — agent does it → AGENTS.md; human does it → `docs/`.
* **Existing docs first**: read and classify before modifying. Migrate accurate content forward; keep unverifiable in place.
* **Migrate then remove**: no deletions until content is classified and accounted for.
* **Progressive disclosure**: AGENTS.md routes; `docs/` provides depth; nested AGENTS.md for modules.
* **Intent-first taxonomy**: classify docs by type (tutorial, how-to, explanation, reference).
* **Validate what you claim**: confirm commands exist; mark ✅/⚠️/⏸️.
* **Deterministic validation**: run `scripts/validate_frontmatter.py` and `scripts/verify_references.py`.
* **Gotchas redirect**: DO X, NOT Y, BECAUSE Z.
* **Invariants are high-stakes**: only discoverable from code/CI. Mark unverified with `REVIEW:`.

Full principles with rationale: `references/principles.md`

Security/ops guardrails:
* Never write secrets into generated files.
* Do not run destructive shell commands.
* Prefer offline, unit-level validation; avoid networked/E2E commands unless explicitly asked.

---

## Inputs

RAW ARGUMENTS: `$ARGUMENTS`

If `$ARGUMENTS` contains `--help` or is exactly `help`, print a usage guide without modifying the repo.

### Parsing

1. Extract flags: `--help`, `--agents`, `--claude`, `--both` (default: `--both`)
2. Parse remaining as: `[mode] [scope...]` (scope can be space/comma separated)
3. Defaults: mode=`auto`, scope=`.`

---

## Modes

**quick-start**: Create from scratch. Ask if scope is ambiguous (monorepo signals). Offer CLAUDE.md consolidation if standalone CLAUDE.md exists.

**update**: Incremental by default. Run `scripts/check_staleness.py` to assess what changed. Only update affected sections. Load `references/incremental-update-workflow.md` for the full algorithm. Falls back to full scan if docs are severely stale or malformed.

**evaluate**: Everything in update, plus evaluate against principles above and propose/apply improvements.

**auto** (default): No docs → quick-start. Has docs → update + evaluate. **Zero user input.** Smart defaults for everything. Automatic CLAUDE.md consolidation, nested doc creation, `docs/` generation for cross-cutting topics. Suitable for CI and background jobs.

---

## Execution Plan

### 1. Determine Root and Scope

* Git root is the anchor (or provided scope directory).
* Detect monorepo signals (workspaces, multiple manifests).
* In auto mode: infer scope from `git status` / `git diff --name-only` if not provided.
* **Cross-cutting system scope**: when the user provides a system name + multiple code paths (e.g., "ML Inference Pipeline" spanning `ml/training/`, `services/triton/`, `api/routes/inference.py`), docs live alongside the primary module. See `references/docs-structure.md` Scale D for details.
* **Scope guardrail**: respect explicit scope — don't create docs in parent/sibling dirs.
* **For update mode**: run `scripts/check_staleness.py` on existing AGENTS.md.

### 2. Discover Authoritative Context

Read (minimal, targeted): README, build/test/lint configs, package manifests, Makefile/Taskfile, CI workflows, repo layout.

### 2b. Discover and Classify Pre-Existing Documentation

**Run this before writing any new files.** Load `references/existing-docs-migration-workflow.md` for the full algorithm.

1. Find non-standard doc directories (`ai_agent_docs/`, `agent-docs/`, `context/`, `llm-docs/`)
2. Find existing `docs/*.md` files without skill-generated YAML frontmatter
3. Read every file found — do not skip based on filename or apparent size
4. Classify each file's content: Architectural (→ `docs/`), Module Operational (→ nested `AGENTS.md`), Fast-Changing (discard), or Unverifiable (keep in place)
5. Verify accuracy of Architectural and Module Operational content (path checks, command existence)
6. Build a migration manifest: source → destination for every piece of content
7. Only proceed to Step 4 after the manifest is complete

**Unverifiable content**: if content cannot be confirmed without running E2E or networked operations, keep it at its current path. Do not delete it. Flag it in the Step 6 report with reason.

**Scope boundary**: This step classifies pre-existing content to inform routing tables
and AGENTS.md updates. Actual file moves, docs/ restructuring, and index creation in
docs/ are owned by the docs-workflow skill. If running both skills, defer docs/ file
operations to docs-workflow. If running context-engineering alone, execute the
migration inline.

### 3. Derive Commands

Typical buckets: install/bootstrap, dev server, test, lint/format, typecheck, build.
Validate each command. For test commands: attempt minimal safe fast execution.
See `resources/generated-doc-spec.md` for detailed validation rules.

### 4. Write or Update Files

Follow `resources/generated-doc-spec.md` for the exact output structure.
All generated `docs/*.md` content files get frontmatter per `resources/frontmatter-index-spec.md`. AGENTS.md files do not get frontmatter.

**Frontmatter rule — applies to created AND updated `docs/*.md` content files:**
* If you create a new docs/ content file: add frontmatter.
* If you update an existing docs/ content file: check if frontmatter exists. If missing, add it.
* "I didn't create it" is not an excuse — if you touched it, it gets frontmatter.
* **Exception — AGENTS.md files at any level are routing indexes, not content docs. They do not get frontmatter.** Agents auto-discover AGENTS.md by convention; frontmatter adds no routing value.

**Intent classification**: when creating `docs/` files, classify each by type (tutorial, how-to, explanation, reference) using the intent classification in `references/docs-structure.md`. Place into intent folders per the structure rules (folders from the start → sub-indexes when a folder hits 8+).

**Docs index generation**: when creating or updating `docs/` files, also create/update `docs/README.md` (human progression index) and `docs/AGENTS.md` (agent routing index). See `references/docs-structure.md` for index specs.

**File strategy:**
* AGENTS.md is canonical. CLAUDE.md is a symlink.
* For update mode: use diff-based editing per `references/incremental-update-workflow.md`.
* Auto mode creates: nested AGENTS.md for modules with distinct workflows, `docs/` files for cross-cutting topics, `docs/architecture.md` for repos with 3+ modules (template in `resources/architecture-doc-template.md`).
* For monorepos: generate cross-module dependency map per `references/cross-module-dependency-map.md`.

### 5. Validate

Run validation scripts, passing the **git root** as scope. The scripts automatically
exclude `_archive/`, `archive/`, `.git`, `.venv`, `node_modules`, `__pycache__`, and
vendored plugin docs (`config/{tool}/plugins/*/docs/`).

```bash
scripts/validate_frontmatter.py {git_root}    # Frontmatter exists, fields valid, ids unique
scripts/verify_references.py {git_root}       # All referenced paths exist
```

**Expected false positives to ignore in the report**:
- `_archive/` files: intentionally archived docs for a different repo/context
- Files you did not create or modify in this run (pre-existing broken refs are not your responsibility)

When reporting validation results, distinguish between:
- **New failures**: broken refs in files you created/modified this run → fix before finishing
- **Pre-existing failures**: broken refs in unmodified files → report as-is, do not fix

In evaluate/auto mode: also apply rubric (conciseness, correctness, progressive disclosure quality). Specifically check that `docs/AGENTS.md` (if it exists) has YAML frontmatter and uses the intent-typed table format specified in `resources/generated-doc-spec.md`. If it predates the skill and lacks frontmatter or uses a different structure, flag it for update.

### 6. Report

Output in chat:
* Files created/updated (paths)
* Key changes (bullets)
* Validation log (commands + ✅/⚠️/⏸️)
* Symlink status (✅ exists / ⚠️ created / missing)
* Script results (frontmatter validation, reference checks)
* **Pre-existing docs inventory** (from Step 2b):
  - Files read (paths + line counts)
  - Migrated: source → destination for each file
  - Kept in place: path + reason (unverifiable / partially verified)
  - Discarded: path + reason + summary of what was in it
  - Source dirs/files removed only after all content accounted for
* Remaining unknowns or follow-ups

Do NOT paste full file contents unless asked.

---

## Scripts

Deterministic validation tools in `scripts/`:

| Script | When to run | What it does |
|--------|-------------|-------------|
| `validate_frontmatter.py {path}` | After generating/updating docs | Checks frontmatter fields, unique ids, heading correspondence |
| `check_staleness.py {agents-md}` | Start of update mode | Git-based staleness assessment, outputs JSON recommendation |
| `verify_references.py {path}` | After generating/updating docs | Confirms all backtick paths and routing table entries exist |

---

## Bundled Content

Load only what the current step requires:

**Resources** (specs and templates for output):

| File | Load when... |
|------|-------------|
| [generated-doc-spec.md](resources/generated-doc-spec.md) | Writing or updating any AGENTS.md (Step 4) |
| [frontmatter-index-spec.md](resources/frontmatter-index-spec.md) | Adding frontmatter to any generated file |
| [architecture-doc-template.md](resources/architecture-doc-template.md) | Generating docs/architecture.md (3+ modules or requested) |

**References** (guides and workflows):

| File | Load when... |
|------|-------------|
| [principles.md](references/principles.md) | Evaluating or writing principles, assessing doc quality |
| [docs-structure.md](references/docs-structure.md) | Creating docs/ files, classifying content by type, structuring directories |
| [incremental-update-workflow.md](references/incremental-update-workflow.md) | Running update mode (Step 1 + 4) |
| [cross-module-dependency-map.md](references/cross-module-dependency-map.md) | Monorepo with 3+ modules |
| [docs-directory-guide.md](references/docs-directory-guide.md) | Creating docs/ overflow files |
| [existing-docs-migration-workflow.md](references/existing-docs-migration-workflow.md) | Pre-existing docs found (Step 2b) — always load when non-standard doc dirs or unfrontmatted docs/*.md exist |
| [deterministic-doc-tests.md](references/deterministic-doc-tests.md) | Suggesting CI integration or deterministic checks for a repo |

---

## Now Execute

Proceed in the parsed mode using the scope rules above.
