---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-10 10:07 - The main context gap was not product behavior

The product surface is much more complete than the onboarding surface. The
recurring rough edges already existed in `.ai/plans/*`, but they were trapped
inside task-local logs instead of being promoted into the repo's durable docs.

## 2026-04-10 10:12 - `.ai/` artifacts were useful as evidence, not as final docs

The plan logs were the right place to mine for repeated failures and design
adjustments: tmux shell portability, atomic native-signal fixtures, Docker or
Colima heavy-gate issues, and harness-specific E2E shape. But those artifacts
should stay historical. The stable repo guidance belongs in `AGENTS.md` and
`docs/`.

## 2026-04-10 10:22 - The repo validator treats `docs/README.md` as a content doc

The generic skill guidance allows `docs/README.md` to skip frontmatter, but
this repo's frontmatter validator expects it. The context pass therefore needed
frontmatter on the docs index too, not just on `tour.md` and `workflows.md`.
