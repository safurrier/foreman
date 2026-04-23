# Implementation Plan

## Test Shape

Use the existing tmux-backed test fixtures:

- `tests/support/tmux.rs` for direct tmux control and dashboard alternate-screen assertions
- existing dashboard binary smoke patterns from `tests/runtime_dashboard.rs`
- existing focus proof patterns from `tests/tmux_focus.rs` and `tests/release_gauntlet.rs`

## Planned Changes

1. Add a deterministic native dashboard smoke test.
   - Start a tmux session with a Claude-like pane.
   - Start Foreman against the fixture socket with an isolated config/log/native dir.
   - Inject Claude hook events using the real `foreman-claude-hook` binary and `TMUX_PANE`.
   - Assert the dashboard shows `Status source: native hook`.

2. Extend the same smoke path to cover attention behavior.
   - Switch the dashboard to `View: attention`.
   - Inject a Claude `Notification` with `permission_prompt`.
   - Assert the dashboard reflects the native attention state in the selected/targeted row.

3. Strengthen the `f` focus proof.
   - Either extend an existing release/runtime smoke path or add a dedicated dashboard-level test that exercises `f` across sessions.
   - Keep the assertion at the tmux active-pane level.

4. Validate.
   - Run targeted tests for the new/changed files first.
   - Run `mise run check`.
   - Run `mise run verify`.

## Risks

- Alternate-screen assertions can be timing-sensitive; prefer waiting on stable dashboard markers that already exist elsewhere in the suite.
- The dashboard text may not literally expose “top of inbox”; use the current `View: attention` model instead.
