---
id: implementation-plan
title: Implementation Plan — Capture Refresh Performance
description: >
  Sequence the low-risk default tuning first, then change tmux preview capture so
  refresh cost scales with operator-visible rows instead of total pane count.
---

# Implementation — capture-refresh-perf

## Current Diagnosis

The bottleneck is not reducer navigation. It is the periodic synchronous refresh:

- `src/runtime.rs`
  - refresh happens in the main loop once `poll_interval_ms` elapses
- `src/adapters/tmux.rs`
  - `load_inventory_profiled()` always does `capture_pane()` for every pane record
- `src/config.rs`
  - defaults are still aggressive for a many-pane world

## Phase 1 — Lower default refresh pressure

### Changes

1. Tune monitoring defaults in `src/config.rs`
   - lower `capture_lines`
   - raise `poll_interval_ms` modestly
2. Update any docs/spec references that mention the default monitoring behavior.
3. Extend the profiling test expectations if needed so the default-config path is
   what gets measured.

### Rationale

This is the fastest safe win. It reduces the size and frequency of refresh spikes
before any structural refactor lands.

### Risks

- Preview text may be slightly shallower by default.
- Slower polling can delay metadata changes if over-tuned.

### Exit criteria

- Defaults still feel fresh enough in dashboard smoke tests.
- Perf traces show smaller `capture_total_ms` than before.

## Phase 2 — Selected/visible-first preview capture

### State/model changes

1. Add enough runtime or app state to know:
   - selected target
   - visible sidebar row targets
   - maybe a rolling cursor for deferred off-screen refresh
2. Introduce cached preview freshness metadata if needed:
   - last captured tick/time
   - stale indicator or source

### Adapter/runtime changes

1. Split tmux inventory refresh into two layers:
   - enumerate pane metadata for all panes
   - capture preview text only for a prioritized subset
2. Prioritize capture for:
   - selected pane
   - visible pane rows
   - panes with no cached preview yet
3. For panes outside that set:
   - reuse the last cached preview in inventory/state
   - schedule a smaller rolling refresh across later ticks
4. Keep timing logs rich enough to show:
   - total pane count
   - captured pane count
   - reused preview count
   - selected/visible capture counts

### Test strategy

1. Unit tests for prioritized capture selection and cache reuse.
2. Runtime smoke proving selected/visible panes stay fresh.
3. Profiling smoke with many sessions proving reduced refresh cost.
4. Full verify + strict native closeout.

### Rationale

This keeps the correctness-critical discovery path current while making preview
capture cost proportional to what the operator is actually using.

### Risks

- Cached previews could become misleading if pane reuse is not reconciled correctly.
- Scroll/selection changes could thrash the priority set if reconciliation is sloppy.
- Existing tests may assume every pane preview is refreshed immediately.

## Deferred option

If Phase 2 still leaves visible hitching, the next design is an async refresh worker
that moves tmux capture off the main loop entirely. That is intentionally deferred
until the lower-risk staged capture design is measured.

## Validation Ladder

### Focused

- config/default tests
- adapter unit tests
- runtime profiling smoke
- runtime dashboard smokes that rely on preview freshness

### Full

```bash
mise run check
mise run verify
mise run native-preflight
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```
