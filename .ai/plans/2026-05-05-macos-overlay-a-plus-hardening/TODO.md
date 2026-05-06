# TODO

## Slice 1 — Hotkey reliability / Ctrl+F fix

- [x] Reproduce/inspect current shortcut state.
- [x] Route recorder/default shortcut through lifecycle-safe Carbon hotkey controller.
- [x] Update hotkey when recorder/reset changes.
- [x] Fix handler cleanup and failed-registration cleanup.
- [x] Validate slice.
- [x] Commit slice.

## Slice 2 — Native text input routing

- [x] Identify current text input interception points.
- [x] Let search/compose text fields own standard editing.
- [x] Preserve type-anywhere search by keeping global typing only outside active text editing.
- [x] Cover navigation vs text input through required overlay gauntlet/manual validation.
- [x] Validate slice.
- [x] Commit slice.

## Slice 3 — Process execution hardening

- [x] Resolve `foreman` path deliberately: bundle, env absolute path, or `/usr/bin/env foreman`.
- [x] Add typed missing executable/launch error handling.
- [x] Make timeout a hard timeout with cleanup.
- [x] Add tests for invalid path, timeout, stderr-only failure, invalid JSON, huge stderr.
- [x] Validate slice.
- [x] Commit slice.

## Slice 4 — Deterministic reloads

- [x] Add reload task cancellation/staleness protection.
- [x] Remove redundant reload-on-show/appear races.
- [x] Add tests for stale response discard.
- [x] Validate slice.
- [x] Commit slice.

## Slice 5 — Selection normalization

- [x] Normalize selection after response/query/filter/sort changes.
- [x] Add tests for selected row hidden by query/filter.
- [x] Validate slice.
- [x] Commit slice.

## Slice 6 — Core.swift module split

- [x] Split models, process runner, client, preferences, store, keyboard reducer, row presenter, terminal preference.
- [x] Preserve public API and behavior.
- [x] Validate slice.
- [x] Commit slice.

## Slice 7 — Validation lane promotion

- [x] Document required overlay validation lane.
- [x] Add/adjust mise task if needed.
- [x] Ensure scripts emit concise summaries/artifacts.
- [x] Validate slice.
- [x] Commit slice.

## Slice 8 — Accessibility/performance cleanup

- [x] Precompute row presentations/duplicate workspace counts.
- [x] Add accessibility labels/non-color status text to critical controls/rows.
- [x] Add tests for row presentation cache behavior.
- [x] Validate slice.
- [x] Commit slice.

## Final

- [x] Run `mise run verify-macos-overlay`.
- [x] Run `mise run install-macos-overlay-app`.
- [x] Mark plan complete.
- [x] Commit plan completion.
