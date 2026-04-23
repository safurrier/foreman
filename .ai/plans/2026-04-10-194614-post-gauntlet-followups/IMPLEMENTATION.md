---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
---

# Implementation — post-gauntlet-followups

## Steps

1. Tighten harness cycling.
   - Teach inventory to report which harnesses are actually present.
   - Make `h` cycle only that ordered subset, then return to `all`.
   - Update reducer tests and compiled-binary gauntlet expectations.

2. Clarify operator affordances.
   - Rewrite preview, input, footer, and help text around `f`.
   - Keep the compact legend, but explain that `f` jumps tmux to the actionable
     pane and that popup mode exits after successful focus.

3. Add explicit keybind contract tests.
   - Cover normal-mode bindings the product advertises.
   - Cover modal bindings for compose, rename, spawn, search, flash, and kill
     confirm.

4. Keep heavy validation intentionally redundant, but make the contract clear.
   - Leave `verify-release` as the report-producing lane.
   - Leave `mise run verify` rerunning it after `cargo test --all-features`.
   - Make the docs and validation notes explicit that this is deliberate release
     proof, not accidental duplication.
   - Add a simple opt-in helper task for real harness E2Es if it improves the
     release drill.

5. Sync docs and validation logs, then run the full stack.
