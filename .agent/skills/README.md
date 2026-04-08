---
id: agent-skills-index
title: Skills
description: >
  Index of optional opinionated workflow plugins for this repo. Each skill encodes
  a repeatable workflow that agents or humans can load on demand.
index:
  - id: adding-a-skill
    keywords: [add-skill, skill-md, references, scripts, structure]
  - id: policy
    keywords: [policy, optional, preference, ci-enforcement, canonical-truth]
---

# Skills

Optional opinionated workflow plugins for agents and humans working in this repo.

Each skill follows this structure:

```
.agent/skills/<skill-name>/
├── SKILL.md          # When to use this skill and the workflow it encodes
├── references/       # Reference materials: context, docs, examples
└── scripts/          # Automation scripts used by the skill
```

## Adding a Skill

A starter template is in `example-skill/SKILL.md`. Copy it:

```bash
cp -r .agent/skills/example-skill/ .agent/skills/<your-skill-name>/
```

Then edit `SKILL.md` to describe:
- When to load this skill (activation signals)
- The opinionated workflow it encodes
- Any references or scripts alongside it

Reference the skill from `AGENTS.md` if it applies broadly.

## Policy

Skills are **preference-based**. Canonical truth stays in `docs/`.

- If a workflow is universally agreed → may become default practice (document in `AGENTS.md`)
- If preference-based → keep as optional Skill
- If critical and objective → encode in CI/tooling instead
