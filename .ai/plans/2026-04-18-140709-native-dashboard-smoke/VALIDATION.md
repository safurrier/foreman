---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

## 2026-04-18

- `cargo test --test runtime_dashboard interactive_binary_surfaces_claude_native_status_and_attention_view -- --nocapture`
  - Failed on the first attempt because the test was asserting against the wrong selected row.
  - Passed after switching to search-driven selection and tightening the expected target marker.
- `cargo test --test runtime_dashboard interactive_binary_popup_focus_action_switches_cross_session_target -- --nocapture`
  - Failed on the first attempt because the test relied on fixed `j` counts and a truncated popup detail pane.
  - Passed after switching to search-driven selection and asserting the live beta workspace target before `f`.
- `cargo test --test runtime_dashboard -- --nocapture`
  - Passed.
- `cargo test --test tmux_focus -- --nocapture`
  - Passed.
- `cargo test --test release_gauntlet -- --nocapture`
  - Passed.
- `mise run check`
  - Failed once on formatting.
  - Passed after `cargo fmt --all`.
- `mise run verify`
  - Passed.
