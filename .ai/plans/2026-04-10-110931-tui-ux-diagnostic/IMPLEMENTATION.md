---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — tui-ux-diagnostic

## Approach

Use a three-part loop:

1. Diagnose the current product live in tmux.
2. Classify failures into information architecture, navigation, discoverability,
   and visual/theming buckets.
3. Turn those failures into a staged implementation plan with validation gates.

The live walkthrough is the source of truth. Code inspection is only for
explaining why a failure happens or where to fix it.

The UX review will use:
- `ratatui-tui` for TUI-specific architecture and focus/mode/layout rules
- `userinterface-wiki` selectively for general UX laws like cognitive load,
  progressive disclosure, and visual emphasis
- `emil-design-eng` selectively for polish, hierarchy, and state clarity

`web-design-foundations` is intentionally not primary here because this is a
terminal UI, not a browser surface.

## Steps

1. Create a repeatable live sandbox:
   a tmux workspace with representative agent panes and the current `foreman`
   binary running against them.
2. Walk through the operator flows manually and capture evidence:
   pane captures, terminal snapshots, and notes tied to concrete actions.
3. Review relevant render/state code only after the live walkthrough:
   identify whether each failure is layout, copy, focus, mode, or missing data.
4. Write a diagnostic summary:
   what is broken, what is merely ugly, what is confusing, and what blocks task
   completion.
5. Turn the diagnostic into implementation slices:
   theme system, information hierarchy, navigation/help/discovery, and surface-
   specific usability improvements.
6. Define validation for each slice:
   render tests, reducer tests, runtime smoke tests, and any visual artifact
   capture we can support.

## Live Findings

### Hierarchy

- Sidebar rows are too generic.
  `Session`, `Window`, and `Pane` consume most of the horizontal space while the
  distinguishing information is either terse (`agents`) or opaque
  (`M1-AFurrier`).
- Harness identity is buried in preview content.
  Operators should not need to enter the preview pane to know whether a target
  is Claude, Codex, Pi, or non-agent.
- Foreman can recursively surface its own dashboard pane as a managed target
  when it is run inside the inspected tmux socket.

### Navigation

- The session tree has no strong expand/collapse affordance.
  `Enter` changes structure, but the UI does not show a clear disclosure state.
- Focus changes are visible only as a border-title star.
  That is technically correct but visually too subtle for fast scanning.
- Search and flash are functional, but the overlays compete with the base layout
  instead of clearly owning a temporary task surface.

### Discoverability

- The help modal is clipped at smaller sizes and omits useful commands like
  `R`, `N`, `H`, `P`, `o`, and `f`.
- The idle input panel does not read as an action surface.
  It says input is available, but does not strongly teach the entry action.
- Toggle state changes are not obvious enough.
  PR detail, filter toggles, and notification profile changes all need stronger
  feedback.

### Functional Gaps Found During UX Review

- Spawn submit behaved inconsistently in the live interactive harness:
  `Ctrl+S` spawned a new window, but `Enter` did not submit the same draft.
- Self-detection is a real bug candidate:
  after filter toggles, the dashboard session appeared as an agent-like target.

## Proposed Slices

### Slice 1: Theme Foundation

- Add a semantic theme module, likely `src/ui/theme.rs`.
- Replace hard-coded `Color::Cyan` / `Color::Yellow` usage with named tokens:
  frame border, focus border, status-attention, status-idle, modal-warning,
  muted text, and emphasis.
- Add a built-in palette set aligned with common TUI defaults:
  catppuccin, gruvbox, tokyo-night, nord, dracula, terminal, and a separate
  no-color fallback mode.
- Add a config seam for theme selection.

Validation:
- render tests asserting themed output still preserves text labels
- theme selection unit tests
- manual `vhs` refresh of the same diagnostic tape under at least one alternate
  theme

### Slice 2: Responsive Layout And Overlay Discipline

- Introduce explicit breakpoints for small, medium, and wide terminals.
- Make overlays clamp to available area instead of bleeding into adjacent panes.
- Rebalance the body layout so preview and input are not permanently locked to a
  72/28 split.
- On smaller terminals, prefer stacked or simplified content over clipped detail.

Validation:
- Ratatui buffer tests at 80x24, 100x30, and 140x40
- live tmux capture at 80x24 using the existing diagnostic harness
- refresh screenshot artifacts

### Slice 3: Sidebar Information Architecture

- Replace generic repeated row labels with denser, more useful labels.
- Surface harness and status earlier in the sidebar.
- Add explicit disclosure markers for collapsed vs expanded sessions.
- Show filter and sort state in a predictable place instead of hiding it behind
  invisible toggles.

Validation:
- render tests for mixed harness inventories
- live tmux capture proving non-agent session/pane toggles remain legible

### Slice 4: Preview And Action Surface Cleanup

- Split preview into a concise operator summary and recent output section.
- Make PR detail state legible even when collapsed.
- Turn the input panel into a clear call-to-action when idle and a proper draft
  editor when active.
- Improve focus signaling beyond the current title-star-only approach.

Validation:
- render tests for session/window/pane selections
- live direct-input smoke test against an interactive shell pane

### Slice 5: Help, Hints, And Keyboard Discoverability

- Rewrite help into categorized sections that fit smaller screens.
- Ensure footer hints are contextual and complete for the active mode.
- Surface hidden-but-important commands (`R`, `N`, `H`, `P`, `o`, `f`) in at
  least one always-available discoverability surface.

Validation:
- buffer tests for help rendering at multiple sizes
- live capture of help/search/flash states
- screenshot refresh using the existing tape

### Slice 6: Functional Fixes Found During Review

- Reproduce and fix the spawn-with-Enter inconsistency.
- Prevent Foreman panes from being recognized as managed AI harness panes.
- Add stronger operator-visible confirmation for toggle actions.

Validation:
- dedicated tmux smoke test for spawn submit via `Enter`
- dedicated tmux smoke test proving Foreman excludes its own pane/session
- runtime test for action-status messaging after toggle commands
