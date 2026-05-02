---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — context-improve-refresh

## Approach

Apply the context-engineering lean default to the current repo. That means
updating the root steering file rather than regenerating the docs tree, because
the docs already exist and have frontmatter.

## Steps

1. Run `context-engineering depth`, `tier`, `sessions`, `references`, and
   `antipatterns` against the repo.
2. Rewrite root `AGENTS.md` into the lean shape: orientation, compact workflow,
   validated commands, gotchas, and related context.
3. Fix broken-reference findings by using real paths or plain prose for
   non-path tokens.
4. Add context-engineering watermarks to generated/routing context files.
5. Re-run context-engineering validation plus `mise run check`.
