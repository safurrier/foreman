---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — default-sort-config

## Approach

Reuse existing typed UI state (`SortMode`, theme, filters, selection, collapsed sessions) and add a small persisted preferences file in Foreman's config area. Apply runtime defaults through a single bootstrap helper after `AppState` construction, because `AppState::with_inventory` immediately rebuilds visible rows. Persist only after intentional operator actions, not every inventory refresh.

For polish work, prefer text/state improvements over new background systems: notification requests should carry better target context, cache controls should reuse the startup cache loader, and PR loading/unavailable copy should be clearer in the existing Details panel.

## Steps

1. Add typed config support for startup sort and popup cache max age.
2. Add persisted UI preferences load/save around runtime actions.
3. Restore sort/theme/filter/collapsed sessions/selection at startup.
4. Apply cache freshness config and expose cache path/age details.
5. Improve notification and PR/cache detail copy.
6. Update README, SPEC, plan artifacts, and logging.
7. Run focused tests, then `mise run check`.
