# Popup Key Latency Validation

## Problem

After enabling all-source popup mode, keypresses could feel sticky on first popup
or while the Coder source refresh merged. Debug logs showed selection and render
were cheap (`move-selection total_ms=0`, `render_frame` usually 0-2ms); the lag
came from background inventory/source work taking event-loop turns while the user
was navigating.

## Before signal from diagnosis

Ad-hoc tmux measurements during diagnosis showed:

- local-only 20-key burst: about 44ms in the best run
- all-source before popup scheduling fixes: about 289ms to 1400ms depending on
  refresh overlap
- source refresh overlap produced slow `inventory_refresh` spans around
  800-1100ms while key input was active

## Implemented mitigation

- Popup drains ready input before background result drains.
- Complete all-source inventory merges are deferred while input happened in the
  last 150ms, then applied after idle.
- Popup automatic PR/extension lookups remain suppressed during navigation.
- Popup all-source periodic refresh remains much slower than the full dashboard.

## Smoke command

```bash
scripts/smoke-popup-key-latency.sh
```

HK evidence `ev_20260608_123301_123823` ran `cargo test --lib`, the ignored
`runtime_profiling` suite, and the smoke together. HK evidence
`ev_20260608_123757_168524` reran the smoke as the TUI/demo proof. HK
evidence `ev_20260608_130141_143207` verified the smoke now enforces budgets
and that the documented `activate_command` jump alias is parsed.

Latest smoke output:

```text
scenario=local-only keys=20 wall_ms=29 action_count=20 slow_actions=0 render_max_ms=1 deferred=0 deferred_apply=0
scenario=all-source-idle keys=20 wall_ms=23 action_count=20 slow_actions=0 render_max_ms=1 deferred=0 deferred_apply=0
scenario=all-source-overlap keys=20 wall_ms=735 action_count=20 slow_actions=0 render_max_ms=2 deferred=1 deferred_apply=1
```

## Acceptance notes

- `slow_actions=0` for local-only, all-source idle, and all-source overlap.
- `deferred=1` and `deferred_apply=1` in the overlap scenario proves the remote
  merge was held until the popup was idle enough to apply it.
- Local-only and all-source idle bursts are in the same order of magnitude; the
  overlap scenario intentionally includes paced key sending so the fake remote
  result lands during navigation.
