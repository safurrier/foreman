# Implementation Plan

## Slice Shape

Keep this as a derived-state and render-path performance pass, not a wider
architecture rewrite.

## Proposed Changes

### 1. Add cached visible operator state

Add a small derived cache to `AppState` that stores:

- base visible targets
- search-filtered visible targets
- sidebar row metadata for those targets
- visible counts used by the header

Rebuild that cache only when inventory, filters, collapse state, sort mode, or
search input changes.

### 2. Make selection movement use cached targets

Update reducer selection movement and related logic to use cached visible
targets instead of rebuilding them for every `MoveSelection`.

### 3. Make sidebar rendering use cached row metadata

Stop recomputing session/window summaries inside `sidebar_line`. Use cached row
metadata and let render only apply selection/flash styling.

### 4. Keep preview/context behavior correct

Leave preview logic mostly intact unless it blocks the performance fix, but make
sure the new cache does not break selection breadcrumbs, workspace path
resolution, diagnostics, or PR display.

### 5. Test and validate

Add focused tests for:

- cache rebuild / selection stability across sort and filter changes
- sidebar rendering still surfacing the expected rows and labels
- any new helper methods introduced by the cache

Then run the full repo validation ladder, including strict native E2Es.
