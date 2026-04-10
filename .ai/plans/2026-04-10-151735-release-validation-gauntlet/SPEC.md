---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — release-validation-gauntlet

## Problem

Foreman has strong targeted tests, but it still lacks one release-grade validation
slice that answers the operator question: "if I boot this in a temporary tmux
world and walk the product, do the important things actually work together?"

The current suite proves many individual seams:

- focused runtime smoke in `tests/runtime_dashboard.rs`
- tmux adapter actions in `tests/tmux_actions.rs`
- inventory/focus/flash coverage in `tests/tmux_inventory.rs`,
  `tests/tmux_focus.rs`, and `tests/tmux_flash.rs`
- notification and pull request flows in `tests/notification_runtime.rs` and
  `tests/pull_requests.rs`
- harness-native bridges and opt-in real-binary E2E for Claude, Codex, and Pi

That is good coverage, but it is not yet a checklist-driven release gauntlet.
There is no single temporary-tmux validation lane that walks the main operator
flows, records what passed, and makes it easy to say which `SPEC.md` acceptance
scenarios are truly proven end to end.

## Requirements

### MUST

- Define a release-validation checklist that maps the important operator-facing
  behaviors in top-level `SPEC.md` acceptance scenarios to concrete proof.
- Build a reusable temporary-tmux scenario harness that can stand up several
  fake agent panes, non-agent panes, workspace repos, and a live Foreman
  dashboard under one socket.
- Add a release-grade automated gauntlet that exercises the most important
  operator flows in the compiled binary, not only reducer or adapter layers.
- Explicitly verify navigation and action fidelity, including selection movement,
  session/window/pane focus resolution, direct input delivery, and popup exit
  behavior.
- Explicitly verify operational discovery flows, including help/legend, theme
  cycling, filter cycling, search, flash navigation, sort changes, and the
  default hiding/toggle behavior for non-agent panes.
- Explicitly verify stateful action flows, including rename, spawn, kill, pull
  request visibility, notification transitions, and graceful degradation when
  optional integrations are unavailable.
- Produce a durable validation artifact or report that makes it obvious which
  checklist items passed and which test command or artifact proved each item.
- Integrate the new gauntlet into the heavy validation story so there is a clear
  release-confidence entrypoint beyond today's focused `verify-ux`.

### SHOULD

- Keep the gauntlet behavior-first and resilient: prefer assertions on actual
  pane focus, pane output, files written, or visible state transitions over
  brittle exact-copy assertions.
- Reuse the existing `TmuxFixture` and test support rather than inventing a
  parallel fixture system.
- Keep real external harness-binary tests opt-in, but clearly connect them to
  the release checklist so native integration proof remains visible.
- Capture artifacts for at least one operator walkthrough so regressions in the
  dashboard can be inspected without reproducing locally.
- Log timing or failure context in a way that helps diagnose slow or flaky
  release-gauntlet failures.

## Constraints

- Do not depend on networked or authenticated agent backends for the main
  release gauntlet. The default release-confidence lane must work with local
  fake agents and temporary repos.
- Avoid turning the gauntlet into one giant opaque test. Prefer a small number
  of scenario-driven tests or subcommands that each prove a coherent slice while
  sharing common setup.
- Do not regress the fast local `mise run check` path. The new work belongs in a
  heavier validation lane.
- Keep tmux setup portable and shell-safe: use `sh`, avoid hard-coded window
  indexes, and continue to tolerate transient tmux startup behavior.
- Maintain sync with `SPEC.md`, `README.md`, `docs/architecture.md`,
  `docs/workflows.md`, and the plan validation log as the new release-confidence
  story takes shape.
