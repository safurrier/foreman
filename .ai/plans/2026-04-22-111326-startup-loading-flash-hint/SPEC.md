---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — startup-loading-flash-hint

## Problem

- Foreman still blocked first paint on the initial tmux inventory sweep.
- In larger tmux worlds, popup startup could take 1–3 seconds before showing anything useful.
- Flash mode also used a blocking popup hint that covered the very rows the operator was trying to jump to.

## Requirements

### MUST

- Render a usable loading frame before the first inventory load completes.
- Move initial inventory population onto the runtime refresh path instead of blocking bootstrap.
- Keep startup preview capture lighter than steady-state refresh so first paint stays fast.
- Remove the blocking flash popup and keep flash guidance non-blocking.
- Preserve existing tmux/dashboard behavior and pass the full validation ladder, including strict real native E2Es.

### SHOULD

- Surface loading state clearly instead of showing an ambiguous empty dashboard.
- Keep flash labels visible inline on the rows they target.
- Update docs and test coverage so the new startup and flash behavior are durable.

## Constraints

- Keep rendering inside the runtime loop instead of introducing a second ad hoc startup renderer.
- Reuse the existing async refresh worker where possible.
- Do not regress release gauntlet, runtime dashboard smokes, or native Claude/Codex/Pi end-to-end coverage.
