---
name: spec-sync
description: >
  Review changes against SPEC.md and docs/decisions/. Capture architectural
  decisions as ADRs, update SPEC.md if invariants changed. Run before pushing.
allowed-tools: Read, Edit, Write, Glob, Grep, Bash
---

Review the current branch's changes against the project's correctness envelope
and decision records. Propose updates if needed, skip if nothing changed.

## When to Run

Before pushing to a PR branch. Part of the pre-push workflow in AGENTS.md:

1. `mise run check` — must pass
2. **`/spec-sync`** — this skill
3. `/context-engineering update` — if AGENTS.md affected
4. `/docs-workflow update` — if docs/ affected

## Process

### 1. Assess what changed

```bash
git diff origin/main...HEAD --name-only
git diff origin/main...HEAD --stat
```

If no changes, report "nothing to sync" and exit.

### 2. Read the spec and existing decisions

- Read `SPEC.md` — current invariants, requirements, interfaces
- Read `docs/decisions/` — existing ADRs (if directory exists)
- Note the highest ADR number for sequential numbering

### 3. Review changes against the spec

For each significant change, ask:

- **Did this change an invariant?** (state rule, safety boundary, interface contract)
  → If yes, update SPEC.md
- **Did this make an architectural decision?** (chose one approach over alternatives,
  established a pattern, added a dependency, changed a boundary)
  → If yes, propose an ADR
- **Did this change a public interface?** (API surface, CLI flags, config format)
  → If yes, update SPEC.md Interfaces section

Skip trivial changes: formatting, typo fixes, test-only changes, dependency bumps
without behavioral impact.

### 4. Write updates

**ADRs**: One ADR per branch summarizing key decisions. Follow the schema:

```markdown
---
id: {project}-adr-{NNNN}
title: ADR {NNNN} — {Title}
description: >
  {One-line summary}
index:
  - id: decision
    keywords: [{relevant, keywords}]
---

# ADR {NNNN}: {Title}

**Status**: Accepted
**Date**: {YYYY-MM-DD}
**Deciders**: {names}
**Generated from**: {diff | pr | agent-session | manual}

---

## Context
{Why this decision was needed}

## Decision
{What was chosen}

## Consequences
**Positive:**
- {benefits}

**Negative / Trade-offs:**
- {costs}

## Alternatives Considered
| Alternative | Reason not chosen |
|---|---|
| {option} | {reason} |
```

**SPEC.md**: Use targeted edits — update only the affected section.
Don't rewrite the whole file.

**Architecture.md decisions index**: If a new ADR was created, add it to the
decisions index table in `docs/architecture.md`.

### 5. Report

Print a summary:
- **No updates needed** — if nothing changed that affects spec/decisions
- **Updated SPEC.md** — list which sections and why
- **Created ADR {NNNN}** — one-line summary of the decision
- **Updated decisions index** — if architecture.md was touched

## Rules

- **Don't over-capture** — not every code change is a decision. Only capture choices
  that constrain future work or that someone would want to understand later.
- **One ADR per branch** — summarize the key decisions, don't create one per commit.
- **Respect existing content** — use targeted edits, never rewrite SPEC.md or
  architecture.md wholesale.
- **Contract tests validate** — `mise run check` will catch malformed ADRs or
  missing SPEC sections. Run it after making changes.
