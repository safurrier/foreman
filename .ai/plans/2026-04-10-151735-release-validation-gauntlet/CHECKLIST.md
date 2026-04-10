---
id: plan-checklist
title: Release Validation Checklist
description: >
  Acceptance-scenario checklist for release-grade operator validation.
---

# Checklist — release-validation-gauntlet

| Scenario | Current proof | Release-gauntlet target |
|---|---|---|
| A1 Config utility flow | `tests/cli_config.rs` | Keep as targeted proof; include in final checklist output |
| A2 Startup logging | `tests/cli_config.rs`, `tests/bootstrap_observability.rs` | Add release artifact check that latest log and run log are created |
| A3 Multi-session discovery | `tests/tmux_inventory.rs` | Cover in startup gauntlet with several sessions under one temp socket |
| A4 Agent detection | `tests/tmux_supported_harnesses.rs` | Cover in startup gauntlet with mixed harness panes and non-agent panes |
| A5 Integration mode selection | `tests/claude_native.rs`, `tests/codex_native.rs`, `tests/pi_native.rs` | Keep targeted proof; record in checklist as native/compat support evidence |
| A6 Default hiding and toggle behavior | `tests/tmux_inventory.rs`, `tests/runtime_dashboard.rs` | Cover in startup gauntlet with explicit operator toggles and harness cycling |
| A7 Status stability | targeted native/compat tests | Add a runtime transition scenario so activity does not flicker in the dashboard |
| A8 Keyboard navigation | `tests/runtime_dashboard.rs` | Expand gauntlet to cover actionable session/window/pane selection, help, theme, and focus resolution |
| A9 Popup auto-exit | `tests/runtime_dashboard.rs` | Keep existing popup flow in gauntlet checklist output |
| A10 Direct input | `tests/runtime_dashboard.rs`, `tests/tmux_actions.rs` | Expand gauntlet to include multiline compose and correct pane delivery |
| A11 Claude native support | `tests/claude_hook_bridge.rs`, `tests/claude_real_e2e.rs` | Keep targeted proof and list it explicitly in release checklist |
| A11b Codex native support | `tests/codex_hook_bridge.rs`, `tests/codex_real_e2e.rs` | Keep targeted proof and list it explicitly in release checklist |
| A11c Pi native support | `tests/pi_hook_bridge.rs`, `tests/pi_real_e2e.rs` | Keep targeted proof and list it explicitly in release checklist |
| A12 Pane operations | `tests/tmux_actions.rs`, `tests/runtime_dashboard.rs` | Add one coherent action gauntlet: focus, rename, spawn, kill |
| A13 Search | reducer/render tests | Add compiled-binary gauntlet coverage for search filter and cancel restore |
| A14 Flash navigation | `tests/tmux_flash.rs` | Add compiled-binary gauntlet coverage for jump and jump+focus flows |
| A15 Sorting | reducer tests only | Add compiled-binary gauntlet coverage for sort cycling and selection preservation |
| A16 Pull request awareness | `tests/pull_requests.rs` | Add runtime gauntlet coverage that surfaces PR state while keeping the dashboard usable |
| A17 Pull request graceful degradation | `tests/pull_requests.rs` | Add runtime gauntlet coverage for missing tooling and soft-failure visibility |
| A18 Notification behavior | `tests/notification_runtime.rs`, `tests/notifications.rs` | Add release-checklist coverage for completion, attention, suppression, and backend order |
| A19 Validation stack | `.mise/tasks/verify`, `.mise/tasks/verify-ux` | Add `verify-release` or equivalent release-confidence lane with artifacts and checklist output |

Completed proof from this slice:

- `tests/release_gauntlet.rs`
- `.mise/tasks/verify-release`
- `.ai/plans/2026-04-10-151735-release-validation-gauntlet/artifacts/release-validation-report.md`
