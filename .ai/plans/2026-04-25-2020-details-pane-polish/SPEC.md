# SPEC — Details pane polish

## Problem

The main Details pane contains the right information, but the hierarchy is dense: alerts, target identity, action hints, setup diagnostics, PR state, and recent output can blur together. Operators need the selected target’s status, source confidence, workspace, and next actions to be easier to scan.

## Requirements

- Keep rendering pure and avoid new runtime side effects.
- Improve the selected target summary near the top of Details.
- Make the selected agent status/source/workspace/action affordances more scannable.
- Preserve existing diagnostics, PR, and recent-output sections.
- Keep compact layouts readable and no-color compatible.

## Acceptance

- Render tests cover the new summary/action copy.
- Existing render/runtime tests still pass.
- `mise run check` passes.
