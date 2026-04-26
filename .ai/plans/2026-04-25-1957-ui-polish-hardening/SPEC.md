# SPEC — UI polish hardening

## Problem

The default-sort/UI-persistence slice works, but review found risks around hot-path disk writes, config precedence, preference corruption visibility, concurrent preference writes, stale cache diagnostics, notification labels, and PR refresh feedback.

## Requirements

- Debounce UI preference writes so navigation does not synchronously write on every move.
- Flush dirty UI preferences before shutdown and log write timing.
- Make UI preference temp files unique per process/write to avoid concurrent instance races.
- Define and implement precedence: explicit `[ui]` config values win over persisted theme/sort; persisted values win only when config omits that key.
- Surface invalid or unreadable `ui-state.json` through runtime diagnostics and `foreman --doctor`.
- Provide a supported reset affordance for persisted UI state.
- Keep stale startup cache readouts useful by reporting last inventory counts.
- Add source/session/window labels to notifications.
- Add manual PR refresh and visible refresh-in-progress feedback.
- Preserve reducer/render purity and keep I/O in runtime/services.

## Acceptance

- Focused tests cover precedence, dirty preference flushing, reset, doctor UI prefs, stale cache readout, notification labels, and PR refresh feedback.
- `mise run check` passes.
