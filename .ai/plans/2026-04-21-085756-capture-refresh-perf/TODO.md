# TODO — capture-refresh-perf

## Planning

- [x] Confirm default tuning target for `capture_lines`
- [x] Confirm default tuning target for `poll_interval_ms`
- [x] Identify current tests that assume full preview capture every tick

## Phase 1 — Quick mitigation

- [x] Lower default `capture_lines`
- [x] Raise default `poll_interval_ms`
- [x] Update docs/spec text that mentions monitoring defaults
- [x] Run focused profiling checks for the new defaults

## Phase 2 — Real fix

- [x] Split full-pane metadata discovery from preview capture policy
- [x] Add selected/visible-first capture prioritization
- [x] Reuse cached previews for off-screen panes
- [x] Add rolling refresh for uncaptured panes
- [x] Extend timing logs with captured-vs-reused preview counts
- [x] Add or update unit tests for prioritized capture behavior
- [x] Add or update runtime smokes for preview freshness
- [x] Add or update profiling smokes for `inventory_refresh`
- [x] Move tmux inventory refresh work off the main loop once staged capture alone proved insufficient

## Validation

- [x] `mise run check`
- [x] `mise run verify`
- [x] `mise run native-preflight`
- [x] Strict `mise run verify-native`

## Closeout

- [x] Update `LEARNING_LOG.md`
- [x] Update `VALIDATION.md`
- [x] Mark `META.yaml` complete
