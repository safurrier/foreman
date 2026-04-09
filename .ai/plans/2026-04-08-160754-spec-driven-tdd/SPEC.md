---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for turning the top-level foreman spec into a
  staged TDD implementation plan.
---

# Specification - spec-driven-tdd

## Problem

`SPEC.md` defines a large product surface, but the current crate is still a
stub. We need an implementation order that lets us build Foreman incrementally,
keep each slice independently testable, and preserve the product and UX
invariants that matter for a keyboard-first Ratatui dashboard.

## Requirements

### MUST

- Treat Foreman as a Ratatui Shape B application: multi-pane, navigation-heavy,
  persistent session.
- Build around `Event -> Command -> Action -> Reducer -> Effects -> Render`.
- Keep render pure and state-driven so most behavior can be tested without tmux.
- Break work into vertical slices that each close one or two acceptance
  scenarios from the root `SPEC.md`.
- Start with pure state, rendering, and adapter seams before real tmux and
  harness I/O.
- Prioritize Claude Code as the first native integration.
- Use four test layers: unit, Ratatui buffer/component, adapter contract, shell
  smoke.
- Enforce terminal-safe UX rules: instant keyboard response, explicit focus,
  visible mode, progressive disclosure, and monochrome-safe status styling.

### SHOULD

- Keep most slices in the 1-2 hour range.
- Make fake-backed tests the default; reserve real tmux flows for smoke tests.
- Add optional surfaces only after the primary shell, navigation, and status
  loop feel stable.
- Keep selection stable across refreshes, filtering, and sorting from the first
  inventory slice onward.

## Constraints

- The current repo has almost no implementation yet, so the first slices must
  establish architecture and test harnesses rather than polishing details.
- `docs/architecture.md` is still mostly a template, so module boundaries will
  need to be written down as they become real.
- Native integrations, pull request lookups, and notification backends must all
  fail soft.
- UI/UX skills are advisory here; CSS-specific guidance should not be forced
  into the TUI.
