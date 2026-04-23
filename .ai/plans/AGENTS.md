# Plans

Each subdirectory is a unit of work with a standard structure. Create one with
`git checkout -b feat/<slug>` then `mise run plan -- <slug>`, or create it manually.

## Required Files

| File | Purpose | When to update |
|------|---------|----------------|
| `META.yaml` | Machine-readable metadata — branch, status, PR, source | At start; update status and PR as work progresses |
| `TODO.md` | Checkable task list | At start; check off as you go |
| `LEARNING_LOG.md` | Dev diary — problems, adaptations, feedback, surprises | Append timestamped entries during work |
| `VALIDATION.md` | How changes were verified, links to artifacts (don't store artifacts here) | After testing; append entries as you verify |

## Optional Files

| File | Purpose | When to create |
|------|---------|----------------|
| `SPEC.md` | Requirements, constraints, scope for this task | Complex or scoped work with multiple requirements |
| `IMPLEMENTATION.md` | Step-by-step approach, design decisions | Non-obvious approach with multiple options |

## META.yaml Fields

```yaml
slug: feature-name           # matches directory suffix
branch: feat/feature-name    # git branch for this work
created: YYYY-MM-DD          # auto-filled by mise run plan
pr:                          # filled when PR opens (number or URL)
status: planned              # planned | in-progress | complete | abandoned
source: feature-request      # freeform — code-review, bug, spike, follow-on, etc.
```

## Lifecycle

```
git checkout -b feat/<slug>  → create a feature branch first
mise run plan -- <slug>      → creates directory with templates
status: planned              → start working
status: in-progress          → append to LEARNING_LOG.md as you go
                             → update VALIDATION.md after testing
/plan-sync before pushing    → verify META, TODOs, logs are current
status: complete             → after merge
```

If a slice changes real harness or native hook behavior, `status: complete`
requires the actual real-harness proof, not a skip-only drill:

```bash
mise run native-preflight
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

`mise run plan` rejects invalid slugs and existing slugs so plan paths stay stable.

## Example

See `_example/` for a complete reference plan showing the progression from
planned through completion with realistic entries.

## What Belongs Here

- Commit structured plan files under `.ai/plans/<slug>/`
- Keep stable validation outputs under `.ai/validation/`
- Do not create committed scratch under `.ai/handoffs/`, `.ai/research/`, or
  `.ai/plans/<slug>/artifacts/`
