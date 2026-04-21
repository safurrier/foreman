---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here.
---

# Validation

## 2026-04-20 22:58 MST

- `cargo test render_help -- --nocapture`
  - PASS
- `cargo test render_displays_help_overlay_with_extended_command_surface -- --nocapture`
  - PASS
- `cargo test load_inventory_profiled_tracks_capture_failures -- --nocapture`
  - PASS
- `cargo test --test runtime_profiling navigation_burst_keeps_move_selection_latency_bounded_with_many_sessions -- --ignored --nocapture`
  - PASS
- `cargo test --test runtime_dashboard interactive_binary_help_scrolls_in_small_layout -- --nocapture`
  - PASS
- `mise run check`
  - PASS
- `mise run verify`
  - PASS
- `mise run native-preflight`
  - PASS
- `FOREMAN_REQUIRE_REAL_E2E=1 FOREMAN_REAL_CLAUDE_E2E=1 FOREMAN_REAL_CODEX_E2E=1 FOREMAN_REAL_PI_E2E=1 mise run verify-native`
  - PASS

## Notes

- The first `mise run check` attempt failed on formatting after the bootstrap logging changes.
  `cargo fmt` fixed that and the rerun passed.
- The first full `mise run check` attempt then failed in
  `interactive_binary_help_scrolls_in_small_layout` because the help overlay now starts with
  `Legend` instead of `Navigate`. Updating that tmux smoke and rerunning the suite resolved the
  regression.
