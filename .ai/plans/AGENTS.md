# Legacy plans

Foreman keeps `.ai/plans/**` as old evidence from the workflow that came before Harness Kit. Never add a new plan here. Don't update an old plan to track current work.

## Current workflow

Run Harness Kit from the active feature worktree:

```bash
hk start <slug> --plan "<intent, constraints, validation>" --target .
hk validate --why "<what this proves>" -- <command>
hk sync --target .
hk ready --target .
```

Commit a small `.ai/hk/<work-id>/` export when a pull request needs a lasting handoff. Keep temporary files in the worktree's HK state instead of a legacy plan.

## Read old plans

- Treat `META.yaml`, `TODO.md`, `LEARNING_LOG.md`, and `VALIDATION.md` as proof for their original task, not current policy.
- Keep old paths and wording unless a task calls for an archive fix.
- Move useful lessons into the current `AGENTS.md` or `docs/` owner.
- Use `.ai/validation/` only for stable checks that the current workflow needs.

The root `AGENTS.md` and `docs/workflows.md` own today's lifecycle rules.

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-08-24 -->
