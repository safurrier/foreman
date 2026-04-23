# Dual Sort + Status Color Pass

## Problem

The operator list already supports `recent` and `attention` views, but the sort
behavior is not communicated as a clear multi-axis model:

- `attention` already behaves like `status → recent`
- `recent` reads like a simple recency sort rather than an explicit compound preset

That makes the sidebar harder to reason about, and the status markers do not yet
match the warmer semantics the operator wants for fast scanning.

## Scope

Plan a narrow slice that:

- makes the operator-list sort presets explicitly multi-axis
- keeps `recent` and `attention` as the product-facing concepts
- updates visible sort labels/help text so the axis order is obvious
- shifts status icon coloring so active/working reads orange, idle green, and error red

## Non-Goals

- arbitrary user-configurable sort builders
- a new view model or another dashboard pane
- changes to lifecycle classification semantics
- provider-specific status remapping
- changing no-color ASCII fallback behavior beyond what readability requires

## Assumptions

- “active” means `AgentStatus::Working`
- the target surface is the operator list/sidebar and the detail legend that explains it
- the existing `attention` mode should stay status-first, recency-second
- the missing behavior is a clearer and more explicit `recent` multi-axis preset, not a fully custom sort UI

## Proposed Outcome

Two explicit operator sort presets:

1. `recent → status`
   - primary: most recent activity first
   - secondary: status rank (`error`, `needs attention`, `working`, `idle`, `unknown`)
2. `status → recent`
   - primary: status rank
   - secondary: most recent activity first

Visible UI should surface that order directly instead of just saying `recent` or
`attention` without explanation.

Color treatment should stay semantic and theme-relative:

- each supported theme keeps its own `working`, `attention`, `idle`, and `error` tokens
- this slice retunes those per-theme tokens toward `working ≈ orange/amber`,
  `attention ≈ yellow`, `idle ≈ green`, `error ≈ red`
- `terminal` should use the closest terminal-safe colors
- `no-color` should stay glyph-and-weight only

## Success Criteria

- Cycling sort mode gives the operator two predictable compound presets instead of one obvious preset and one vague one.
- Selection remains stable when sort mode changes.
- The footer/help/context surfaces explain the current sort order in a short way.
- Working/active status markers render with a warm orange-like semantic color in supported themes.
- Idle remains green and error remains red across the supported color themes.
- No palette-backed theme regresses into a single hard-coded global status color set.

## Key Files

- `src/app/state.rs`
  - sort mode model, rank comparisons, and visible-target ordering
- `src/app/action.rs`
  - sort cycling behavior if the mode list changes
- `src/ui/render.rs`
  - header/footer/help wording and status legend
- `src/ui/theme.rs`
  - semantic status colors and glyph semantics
- `src/app/reducer.rs`
  - selection preservation expectations
- `tests/release_gauntlet.rs`
  - possible compiled-binary sort proof if the UX wording changes enough to justify it

## Risks

- If the labels get too long, the header/footer can become noisy in compact layouts.
- Changing named theme palettes needs a small audit so “orange active” stays legible on each background.
- If the current user intent is actually “make sort fully configurable,” this slice will be intentionally smaller than that.
