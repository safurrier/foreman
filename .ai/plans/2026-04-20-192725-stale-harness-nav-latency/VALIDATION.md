---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here.
---

# Validation

## 2026-04-20

- Focused diagnosis and smoke:
  - `cargo test --test runtime_profiling -- --ignored --nocapture`
  - `cargo test --test runtime_dashboard -- --nocapture`
  - `cargo test --test notification_runtime -- --nocapture`
  - `cargo test --test release_gauntlet -- --nocapture`
- Full validation ladder:
  - `mise run check`
  - `mise run verify`
  - `mise run native-preflight`
  - `FOREMAN_REQUIRE_REAL_E2E=1 FOREMAN_REAL_CLAUDE_E2E=1 FOREMAN_REAL_CODEX_E2E=1 FOREMAN_REAL_PI_E2E=1 mise run verify-native`
- Additional native debugging passes after the first strict failure:
  - `FOREMAN_REAL_CLAUDE_E2E=1 cargo test --test claude_real_e2e -- --ignored --nocapture`
  - `FOREMAN_REAL_PI_E2E=1 cargo test --test pi_real_e2e -- --ignored --nocapture`

Result:
- All commands above passed on final `HEAD`.
- The strict real native drill passed after converting the Claude and Pi prompt
  loops from shell-backed fixtures to non-shell Python wrappers.
