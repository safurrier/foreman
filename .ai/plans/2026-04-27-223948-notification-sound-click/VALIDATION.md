---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

## 2026-04-27

- `cargo test services::notifications --lib` - passed.
- `cargo test config --lib` - passed.
- `cargo test --test notifications --test notification_runtime` - passed.
- `mise run check` - passed.
- `cargo run -- --config-show | sed -n '/notifications:/p'` - confirmed local
  config resolves `sound_profile=itysl` and `backends=alerter, notify-send,
  osascript`.
- `cargo test services::notifications::tests::alerter_click --lib` - passed
  after adding a real isolated tmux-server validation for notification
  click-through.
- `cargo test --test notifications --test notification_runtime` - passed after
  the click-through fix.
- `mise run check` - passed after the click-through fix.
- `mise run install-local` - installed the fixed local binaries.
- Added alerter/tmux click diagnostics to `latest.log`, then reran
  `cargo test services::notifications::tests::alerter_click --lib`,
  `cargo test --test notifications --test notification_runtime`, `mise run
  check`, and `mise run install-local` successfully.
