---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

- 2026-04-12 12:31 - `FOREMAN_REAL_CODEX_E2E=1 cargo test --test codex_real_e2e -- --ignored --nocapture` failed with `codex-cli 0.92.0` because no `submit` hook fired.
- 2026-04-12 12:36 - `FOREMAN_REAL_PI_E2E=1 cargo test --test pi_real_e2e -- --ignored --nocapture` passed.
- 2026-04-12 12:37 - `FOREMAN_REAL_CLAUDE_E2E=1 cargo test --test claude_real_e2e -- --ignored --nocapture` failed waiting for the completion notification artifact.
- 2026-04-12 12:39 - Updated the PATH-resolved Codex install to `0.120.0` and reran `FOREMAN_REAL_CODEX_E2E=1 cargo test --test codex_real_e2e -- --ignored --nocapture`; it passed.
- 2026-04-12 12:54 - `python3 -m py_compile .mise/tasks/native-preflight .mise/tasks/verify-native` passed.
- 2026-04-12 12:56 - `mise run native-preflight` passed after adding the new task. Pi auth remains advisory there by design; the real Pi E2E remains the source of truth.
- 2026-04-12 12:57 - `FOREMAN_REQUIRE_REAL_E2E=1 mise run verify-native` failed as intended when the required env flags were not set.
- 2026-04-12 13:16 - `FOREMAN_REAL_CLAUDE_E2E=1 cargo test --test claude_real_e2e -- --ignored --nocapture` passed after making the helper pane visible in the agent view.
- 2026-04-12 13:16 - `FOREMAN_REAL_CODEX_E2E=1 cargo test --test codex_real_e2e -- --ignored --nocapture` passed with Codex `0.120.0`.
- 2026-04-12 13:17 - `mise run native-preflight` passed on the edited tree.
- 2026-04-12 13:18 - `FOREMAN_REQUIRE_REAL_E2E=1 FOREMAN_REAL_CLAUDE_E2E=1 FOREMAN_REAL_CODEX_E2E=1 FOREMAN_REAL_PI_E2E=1 mise run verify-native` passed.
- 2026-04-12 13:21 - `mise run check` passed.
- 2026-04-15 13:45 - `mise run native-preflight` passed on the final tree after the later setup/doctor changes.
- 2026-04-15 13:45 - `FOREMAN_REQUIRE_REAL_E2E=1 FOREMAN_REAL_CLAUDE_E2E=1 FOREMAN_REAL_CODEX_E2E=1 FOREMAN_REAL_PI_E2E=1 mise run verify-native` passed again on the final tree.
