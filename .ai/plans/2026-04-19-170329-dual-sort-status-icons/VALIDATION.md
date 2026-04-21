---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

- 2026-04-19 MST
  - `cargo test release_action_gauntlet_proves_search_flash_sort_and_pane_operations -- --nocapture`
    - PASS
  - `cargo test interactive_binary_surfaces_claude_native_status_and_attention_view -- --nocapture`
    - PASS
  - `cargo test recent_sort_uses_attention_rank_as_tiebreaker -- --nocapture`
    - PASS
  - `cargo test attention_sort_uses_recent_activity_as_tiebreaker -- --nocapture`
    - PASS
  - `cargo test ui::render::tests:: -- --nocapture`
    - PASS
  - `mise run check`
    - PASS
  - `mise run verify`
    - PASS
  - `mise run native-preflight`
    - PASS
  - `FOREMAN_REQUIRE_REAL_E2E=1 FOREMAN_REAL_CLAUDE_E2E=1 FOREMAN_REAL_CODEX_E2E=1 FOREMAN_REAL_PI_E2E=1 mise run verify-native`
    - PASS
