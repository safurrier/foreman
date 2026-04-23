---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — release-validation-gauntlet

## Approach

Treat this as a validation-product slice, not a test-file cleanup.

The deliverable is a release-confidence lane with three layers:

1. A feature checklist tied back to `SPEC.md` acceptance scenarios.
2. A reusable temporary-tmux scenario harness for compiled-binary walkthroughs.
3. A heavy validation entrypoint that runs the gauntlet, captures artifacts, and
   produces a human-readable pass/fail trail.

The important design decision is to keep the main gauntlet local and simulated:
fake agent panes, fake repos, fake notification backends, and a real Foreman
binary. Native real-binary harness tests for Claude/Codex/Pi stay opt-in and
remain supporting proof, not the default release gate.

## Steps

1. Build a release checklist and coverage matrix.
   - Enumerate the operator-visible feature surface from `SPEC.md` acceptance
     scenarios A3 through A19.
   - Map current proof to existing tests.
   - Mark gaps where there is no compiled-binary or cross-feature proof.

2. Refactor the tmux test support into a scenario builder.
   - Add helpers for creating harness-tagged panes, non-agent panes, repo-backed
     panes, notification backends, and dashboard sessions.
   - Add reusable assertions for:
     - active pane changed as expected
     - input arrived in the target pane
     - expected pane/window/session became visible or hidden
     - expected file/log entry was written

3. Add a release-gauntlet runtime suite.
   - Slice 1: startup, discovery, filters, help, theme, and navigation
   - Slice 2: focus, compose, rename, spawn, kill, search, flash, and sort
   - Slice 3: PR surface, notification transitions, graceful degradation, and
     observability checks
   - Keep each test coherent and scenario-based, not microscopic.

4. Add a reporting layer.
   - Generate a Markdown or JSON checklist artifact that records the scenario,
     the proof command, and the pass/fail result.
   - Save artifact paths in the plan validation log.

5. Wire a heavy validation task.
   - Introduce a dedicated release-confidence task such as
     `mise run verify-release`, or extend `verify-ux` in a way that preserves
     clarity.
   - Ensure `mise run verify` either includes the new lane or clearly points to
     it as the release gate.

6. Sync operator docs and release guidance.
   - Document the new validation entrypoint.
   - Document when to run the opt-in real-binary harness tests.
   - Keep the PR/release checklist consistent with the new gauntlet.
