---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
---

# Specification — post-gauntlet-followups

## Problem

The release gauntlet closed the broad product-confidence gap, but it also made a
few smaller operator problems obvious:

- harness cycling still lands on empty views for supported providers that are
  not present in the current tmux world
- the UI says `f tmux focus`, but does not explain clearly enough that this
  jumps tmux to the actionable pane, even from a session or window row
- keybind coverage is behaviorally broad, but not expressed as an explicit
  advertised-key contract
- `mise run verify` currently reruns the release gauntlet after
  `cargo test --all-features`, which wastes time in the heavy lane

The follow-up slice needs to tighten those seams without changing the product
shape.

## Requirements

### MUST

- Harness filter cycling skips empty harness views by default and returns to
  `all` after the last harness that is actually present in inventory.
- The operator-facing copy explains tmux focus in plain language, especially
  when the selected row is a session or window that resolves to an actionable
  pane.
- Advertised keybinds are covered by explicit command-mapping tests rather than
  only indirect behavior tests.
- The compiled-binary release proof stays in place and remains green after the
  focus and harness-cycle changes.
- `mise run verify` keeps the report-producing release gauntlet in the heavy
  lane even though broader Rust test coverage already exercised the same code
  paths, and the docs make that intentional redundancy clear.
- Repo truth stays synced:
  - `SPEC.md`
  - `README.md`
  - `docs/architecture.md`
  - `docs/workflows.md`
  - this plan trail

### SHOULD

- The help surface should tell the operator that `h` cycles only visible
  harnesses by default.
- There should be a documented pre-release command surface for the opt-in real
  Claude, Codex, and Pi E2Es.
- Focus behavior should be proven on session, window, and pane selections, not
  only described.

## Constraints

- Keep the fast gate untouched. The added proof belongs in unit tests or the
  existing heavier compiled-binary lanes.
- Do not add a second filter mode just to preserve empty harness views unless it
  is actually needed. Default operator ergonomics matter more than exhaustive
  provider cycling.
- Keep help text compact enough for the existing terminal breakpoints.
