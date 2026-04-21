---
id: plan-validation
title: Validation Log
description: >
  How changes were verified.
---

# Validation

- Focused Rust coverage:
  - `cargo test app::reducer::tests:: -- --nocapture`
  - `cargo test ui::render::tests:: -- --nocapture`
  - `cargo test --test release_gauntlet release_startup_navigation_gauntlet_proves_discovery_filters_and_help -- --nocapture`
- Full validation ladder:
  - `mise run check`
  - `mise run verify`
  - `mise run native-preflight`
  - `FOREMAN_REQUIRE_REAL_E2E=1 FOREMAN_REAL_CLAUDE_E2E=1 FOREMAN_REAL_CODEX_E2E=1 FOREMAN_REAL_PI_E2E=1 mise run verify-native`
- Result:
  - All commands passed on the final tree.
  - `mise run verify` refreshed the existing UX GIF/screenshots and release report artifacts under the older validation plan directories.
