# j/k Navigation Performance

## Problem

Sidebar navigation feels laggy when moving up and down the operator list with
`j` and `k`, especially when many tmux targets are visible.

The current path recomputes the visible target tree and re-sorts windows and
panes repeatedly:

- during `MoveSelection`
- again during header/sidebar render
- again inside per-row sidebar summaries

That makes simple cursor movement do much more work than necessary.

## Scope

This slice should:

- make repeated `j` and `k` navigation cheap when inventory/filter/search state
  has not changed
- remove obvious repeated visible-tree walks from the sidebar render path
- preserve current operator behavior and selection semantics
- keep all existing compiled-binary and real-provider validations green

## Non-Goals

- redesign the information architecture
- change lifecycle semantics or provider behavior
- add arbitrary user-configurable performance knobs
- fully redesign preview/context caching unless required for the fix

## Desired Outcome

- `MoveSelection` uses cached visible operator targets instead of rebuilding the
  tree on every keypress
- sidebar rendering reuses cached row summaries instead of recomputing visible
  window/pane counts and marks per row on every frame
- selection changes remain logically correct across search, collapse, sort, and
  filter changes
- the current release/runtime smokes still prove focus, attention view, and
  compiled-binary flows

## Risks

- stale cache bugs if inventory/filter/search mutations do not invalidate the
  derived view correctly
- search, flash labels, and collapse behavior can regress if the cache boundary
  is too narrow
- render changes that only optimize one surface may leave another repeated walk
  on the hot path
