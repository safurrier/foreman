---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — context-improve-sync

## Approach

Treat this as a targeted context-improve pass, not a product-code slice.
Read the current steering files and the `.ai/` plan history, identify repeated
workflow or environment lessons, then promote only the durable ones into stable
docs.

Use this split:
- `AGENTS.md` for short steering and high-impact gotchas
- `docs/tour.md` for repo onboarding
- `docs/workflows.md` for plan/validation/environment rules
- `docs/README.md` and `docs/AGENTS.md` as indexes

## Steps

1. Inventory `AGENTS.md`, `README.md`, `docs/`, and `.ai/` artifacts.
2. Distill repeated rough edges that should not remain buried in plan logs.
3. Rewrite root `AGENTS.md` so it reflects the real repo layout and steering rules.
4. Add `docs/README.md`, `docs/AGENTS.md`, and at least one quickstart-style doc.
5. Validate references and frontmatter, then record the results in the plan.
