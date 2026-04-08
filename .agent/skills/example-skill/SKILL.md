---
id: example-skill
title: Example Skill
description: >
  Starter template for a workflow skill. Copy this directory, rename it,
  and fill in your opinionated workflow. Delete this when you have real skills.
index:
  - id: activation
    keywords: [when-to-load, signals, triggers, activation]
  - id: workflow
    keywords: [steps, procedure, commands, how-to]
  - id: references
    keywords: [docs, scripts, resources, context]
---

# Example Skill

> Copy `.agent/skills/example-skill/` to `.agent/skills/<your-skill-name>/`,
> update this file, and add supporting files to `references/` or `scripts/`.

## Activation

Load this skill when: <!-- describe the specific task or signal that triggers this skill -->

**Signals** — keywords that indicate this skill applies:

- <!-- signal-one: e.g., "writing a new API endpoint" -->
- <!-- signal-two: e.g., "adding a database migration" -->

## Workflow

<!-- The opinionated, step-by-step workflow this skill encodes -->

1. <!-- First step -->
2. <!-- Second step -->

### Key commands

```bash
mise run check   # always verify before committing
```

## References

Supporting files for this skill (loaded on demand):

- `references/` — additional context: specs, examples, prior art
- `scripts/` — automation scripts used by this workflow
