# Legacy plans

The repository retains `.ai/plans/**` as historical evidence from the pre-Harness Kit workflow. Never create a new plan here or update an old plan to track current work.

## Current workflow

Use Harness Kit from the active feature worktree:

```bash
hk start <slug> --plan "<intent, constraints, validation>" --target .
hk validate --why "<what this proves>" -- <command>
hk sync --target .
hk ready --target .
```

Commit a compact `.ai/hk/<work-id>/` export when the PR needs a durable handoff package. Keep transient artifacts under the worktree-local HK state rather than copying them into a legacy plan.

## Historical interpretation

- Treat `META.yaml`, `TODO.md`, `LEARNING_LOG.md`, and `VALIDATION.md` as evidence for their original slice, not current repository policy.
- Preserve historical paths and wording unless the task explicitly requires an archival correction.
- Promote recurring lessons into current `AGENTS.md` or `docs/` owners.
- Use `.ai/validation/` only for stable repository validation roots that current workflow depends on.

The root `AGENTS.md` and `docs/workflows.md` own current lifecycle policy.

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-08-24 -->
