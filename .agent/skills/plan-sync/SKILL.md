---
name: plan-sync
description: >
  Verify plan artifacts are current before pushing. Checks META.yaml status,
  TODO completion, and that LEARNING_LOG and VALIDATION have entries.
allowed-tools: Read, Edit, Glob, Grep, Bash
---

Verify that the active plan directory is up to date before pushing.

## When to Run

Before pushing. Part of the pre-push workflow in AGENTS.md:

1. `mise run check` — must pass
2. **`/plan-sync`** — this skill
3. `/spec-sync` — capture decisions, update SPEC.md if invariants changed
4. `/context-engineering update` — update AGENTS.md if needed
5. `/docs-workflow update` — update docs/ if needed

## Process

### 1. Find the active plan

```bash
# Find plan directories, sorted by most recent
ls -d .ai/plans/20*/ 2>/dev/null | sort -r | head -5
```

Look for plans with `status: in-progress` or `status: planned` in META.yaml.
If no active plan exists, report "no active plan" and exit.

### 2. Check META.yaml

- `status` should reflect reality:
  - Has commits on branch? → should be `in-progress`, not `planned`
  - PR open? → `pr` field should be filled in
  - All TODOs done? → consider `complete`
- `branch` should match current git branch

### 3. Check TODO.md

- Are completed items `[x]` consistent with actual changes?
- Are there unchecked items that appear to be done (files exist, tests pass)?
- Any items that should be added based on work done but not tracked?

### 4. Check LEARNING_LOG.md

- If work has been done (commits exist), there should be at least one entry
- Recent entries should be timestamped

### 5. Check VALIDATION.md

- If tests have been run, there should be at least one entry
- Should reference `mise run check` results at minimum

### 6. Report

- **Plan is current** — if everything checks out
- **Updates needed** — list what should be updated, propose edits

## Rules

- **Don't block on empty optional files** — SPEC.md and IMPLEMENTATION.md are optional
- **Suggest, don't rewrite** — propose specific edits, let the agent apply them
- **Lightweight** — this should take seconds, not minutes
