---
id: plan-spec
title: Capture Refresh Performance
description: >
  Reduce the periodic tmux refresh hitch that still makes j/k feel less snappy
  under a crowded session set.
---

# Specification — capture-refresh-perf

## Problem

The main `j/k` hot path is now cheap, but the dashboard still hitches during
inventory refresh because tmux capture work runs synchronously on the main loop.
Live traces showed:

- `move-selection`: effectively `0 ms`
- `render_frame`: roughly `1–2 ms`
- `inventory_tmux`: roughly `190–286 ms`
- most refresh cost coming from `capture-pane`, not `list-panes`

The current defaults make that worse:

- `poll_interval_ms = 1000`
- `capture_lines = 200`

That means Foreman re-captures every pane every second, even when the operator is
only looking at a small subset of rows.

## Requirements

### MUST

- Reduce default refresh pressure so a fresh install feels better without tuning.
- Keep pane discovery fresh enough that selection, focus, lifecycle, and alerts do
  not become stale or misleading.
- Stop doing full `capture-pane` work for every pane on every refresh tick.
- Prioritize preview freshness for the selected row and currently visible sidebar rows.
- Preserve current native overlay behavior, compatibility fallback, attention sorting,
  tmux focus, and real-harness E2E behavior.
- Add validation that measures refresh behavior under a crowded tmux fixture instead
  of relying only on `move-selection` timings.
- Run the full validation ladder, including strict native E2Es, before calling the
  slice done.

### SHOULD

- Keep tmux metadata refresh (`list-panes`) separate from preview capture policy.
- Refresh non-visible pane previews gradually so previews do not stay stale forever.
- Make any staleness explicit in the internal model and logs so debugging stays easy.
- Preserve current log-based profiling so local lag can still be diagnosed after the fix.

## Constraints

- Do not introduce a correctness regression where dead or reused panes linger forever.
- Do not block focus, input, kill, spawn, or rename actions on background capture work.
- Do not make the render path heavier to compensate for lighter refresh work.
- Avoid a large async architecture rewrite unless the staged capture strategy still
  fails the perf budget.

## Non-Goals

- Flattening the topology sidebar into a different IA.
- Replacing tmux as the source of truth.
- Perfectly real-time preview text for every off-screen pane.
- Reworking provider adapters or notification semantics beyond what refresh policy
  needs for correctness.

## Phase Split

### Phase 1 — Quick mitigation

- Lower default `capture_lines` into the `20–50` range.
- Raise the default inventory poll interval modestly so refresh spikes happen less often.
- Add perf validation for the new defaults.

### Phase 2 — Real fix

- Keep full tmux metadata discovery on each refresh tick.
- Capture only the selected pane plus the rows visible in the sidebar viewport first.
- Reuse the last known preview for off-screen panes.
- Gradually refresh off-screen panes across later ticks instead of capturing them all
  at once.
- If this still fails the perf budget, the follow-on option is an async refresh worker,
  but that is not the first design for this slice.

## Acceptance Criteria

- With the default config, crowded-session navigation no longer shows large periodic
  hitches from `inventory_refresh`.
- A profiling smoke proves:
  - lower `inventory_tmux total_ms`
  - lower `capture_total_ms`
  - no regression in `move-selection` latency budgets
- Selected and visible rows keep current preview text reasonably fresh during movement.
- Off-screen panes eventually get refreshed without starving forever.
- `mise run verify` passes.
- `mise run native-preflight` passes.
- Strict real-harness closeout passes:

```bash
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```
