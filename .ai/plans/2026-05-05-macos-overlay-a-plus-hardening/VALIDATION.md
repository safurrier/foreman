# Validation

| Slice | Swift tests | Full verifier | Install | Notes |
|---|---|---|---|---|
| 1 Hotkey reliability | passed | passed | passed | Replaced KeyboardShortcuts handler path with Foreman's lifecycle-safe Carbon controller for recorder/default shortcuts; status now reflects actual Carbon registration. |
| 2 Native text input | passed | passed | passed | Local key monitor now passes normal text editing events through to AppKit text fields instead of mutating query/compose state manually while editing. |
| 3 Process execution | passed | passed | passed | Added typed missing executable/launch errors, `/usr/bin/env` PATH fallback for bare executables, readability-handler pipe draining, hard timeout cleanup, and stderr/timeout/invalid JSON tests. |
| 4 Deterministic reloads | passed | passed | passed | Reloads now cancel/replace prior tasks, use a generation guard to discard stale completions, and panel show no longer races with OverlayView appear reload. |
| 5 Selection normalization | passed | passed | passed | Selection now normalizes after response/query/filter/sort changes so selectedEntry and List(selection:) stay aligned. |
| 6 Core.swift split | passed | passed | passed | Split 1k-line Core.swift into focused Models/ProcessRunner/ForemanClient/Preferences/Store/Keyboard/RowPresenter/Terminal files without public API changes. |
| 7 Validation lane | passed | passed | passed | Added `mise run validate-macos-overlay-change` required lane alias and updated validation docs with current hotkey/default/gauntlet state. |
| 8 Accessibility/performance | passed | passed | passed | Row presentations now batch duplicate workspace counts once per visible list, and rows/status badges expose non-color accessibility labels. |
| Final | passed | passed | passed | Ran required verifier and installed/relaunched Foreman.app. |
