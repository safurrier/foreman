# SPEC — Sidebar Tree With Quiet Singletons

## Problem

The previous singleton-window elision reduced row noise, but it also weakened the left sidebar's file-tree mental model. The operator preferred the earlier explicit hierarchy because it makes tmux topology intuitive and scannable.

## Requirements

- Preserve the visible `session -> window -> pane` tree in the sidebar.
- Keep singleton metadata quiet by hiding redundant `1w/1p` and `1p` labels.
- Preserve all selection/action behavior for session, window, and pane rows.
- Keep multi-pane window counts visible.
- Keep no-color and flash navigation behavior intact.
- Update `SPEC.md`, regression tests, runtime tests, and UX artifacts.

## Non-Goals

- No large visual redesign beyond restoring tree topology.
- No changes to tmux inventory discovery or harness detection.


## Popup Consistency Addendum

- Typical popup dimensions should preserve the same side-by-side sidebar/details layout as regular dashboard mode.
- `verify-ux` should capture popup mode as a visual artifact, not just regular mode.
