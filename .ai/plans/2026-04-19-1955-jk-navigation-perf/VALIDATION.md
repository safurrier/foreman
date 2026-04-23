---
id: plan-validation
title: Validation Log
description: >
  How changes were verified.
---

# Validation

- Focused Rust coverage:
  - `cargo test ui::render::tests:: -- --nocapture`
  - `cargo test app::state::tests:: -- --nocapture`
  - `cargo test move_selection_uses_sorted_visible_targets -- --nocapture`
  - `cargo test --test runtime_profiling -- --ignored --nocapture`
  - `cargo test interactive_binary_surfaces_claude_native_status_and_attention_view -- --nocapture`
  - `cargo test interactive_binary_popup_focus_action_switches_cross_session_target -- --nocapture`
  - `cargo test release_action_gauntlet_proves_search_flash_sort_and_pane_operations -- --nocapture`
- Fast gate:
  - `mise run check`
- Heavy verification:
  - `mise run verify`
- Native closeout:
  - `mise run native-preflight`
  - `FOREMAN_REQUIRE_REAL_E2E=1 FOREMAN_REAL_CLAUDE_E2E=1 FOREMAN_REAL_CODEX_E2E=1 FOREMAN_REAL_PI_E2E=1 mise run verify-native`
- Result:
  - All commands passed on the final tree.
  - `mise run verify` refreshed the existing UX GIF/screenshots and release report artifacts under the older validation plan directories.
