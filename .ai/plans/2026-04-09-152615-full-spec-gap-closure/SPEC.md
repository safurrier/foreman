---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — full-spec-gap-closure

## Problem

The interactive runtime and core operator flows are now in place, but the
top-level `SPEC.md` still promises a broader public CLI/config surface than the
implementation exposes today. The main remaining 1.0 gaps are not in tmux
control or Ratatui rendering; they are in public logging/configuration
contracts and in making the heavy validation loop practical and repeatable.

## Requirements

### MUST

- Keep the shipped runtime loop, tmux flows, and current smoke suite green
  while closing the remaining public-spec gaps.
- Add a public CLI debug logging mode so the CLI contract matches the top-level
  spec.
- Extend user config so notification behavior is not hardcoded:
  notification cooldowns, backend preference/order, and active profile must be
  configurable with safe defaults.
- Add per-harness integration preference where multiple modes are possible.
  For 1.0, Claude must honor config that prefers native, compatibility, or
  automatic selection.
- Thread the new config through runtime/services without moving logic into the
  renderer or reducer.
- Keep `mise run check` green and `mise run verify` green after the changes.
- Update `SPEC.md`, `README.md`, `docs/architecture.md`, and validation logs so
  the repo truth matches the actual runtime and config surface.

### SHOULD

- Keep config evolution backward-compatible and additive.
- Prefer a small set of built-in named notification profiles over a
  user-authored free-form profile language unless the implementation clearly
  needs more.
- Reduce Docker build context size so heavy validation stays fast enough to run
  regularly on a developer machine.
- Preserve the current `Command -> Action -> Reducer -> Effects -> Render`
  boundary while adding configuration.

## Constraints

- Do not weaken the existing tmux smoke tests to make the new config work.
- Do not add reducer-side I/O or logger-side business decisions.
- Do not remove compatibility-mode support for non-Claude harnesses.
- Do not redefine the top-level spec downward just to match the current code;
  this plan assumes the goal is to meet the original 1.0 contract.

## Remaining Gap Checklist

- [ ] CLI debug logging mode exists and is documented.
- [ ] Notification cooldown is configurable instead of hardcoded.
- [ ] Notification backend preference/order is configurable.
- [ ] Active notification profile can be selected from config defaults.
- [ ] Claude integration preference can be forced to native, compatibility, or
  automatic selection.
- [ ] Heavy validation is still green after the config surface expands.
