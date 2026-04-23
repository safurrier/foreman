---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — context-improve-sync

## Problem

The repo now ships the full 1.0 product surface, but the durable onboarding
material lagged behind the implementation and the `.ai/` history. The root
`AGENTS.md` still looked like scaffold output, `docs/` lacked a human and agent
index, and repeated operational lessons from the plan logs were easy to miss
because they only lived in `.ai/plans/*`.

## Requirements

### MUST

- Promote recurring workflow and validation lessons from `.ai/` artifacts into
  stable docs or steering files.
- Replace inaccurate or placeholder-style root context with repo-specific
  guidance.
- Add a quickstart-style docs entry point for future sessions.
- Keep plan logs as evidence, but not as the only place a recurring lesson is
  recorded.
- Preserve the existing correctness hierarchy: `SPEC.md` for product contract,
  `docs/architecture.md` for system design, `AGENTS.md` for short steering, and
  `docs/` for deeper workflow guidance.

### SHOULD

- Add human and agent docs indexes under `docs/`.
- Capture the highest-value non-code rough edges:
  tmux shell portability, atomic native-signal fixtures, validation layering,
  and Docker/Colima heavy-gate behavior.
- Keep the new docs compact enough to stay maintainable.

## Constraints

- Do not rewrite historical plan logs to look cleaner than they were.
- Do not duplicate long architecture content that already lives in
  `docs/architecture.md`.
- Keep root `AGENTS.md` concise and routing-oriented.
