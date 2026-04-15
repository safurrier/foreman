# Release Validation Report

- Generated: `2026-04-15T22:44:15.594871+00:00`
- Status: `PASS`
- Checklist: `.ai/plans/2026-04-10-151735-release-validation-gauntlet/CHECKLIST.md`
- Raw output: `.ai/plans/2026-04-10-151735-release-validation-gauntlet/artifacts/release-gauntlet-output.txt`

## Command

```bash
cargo test --test release_gauntlet -- --test-threads=1 --nocapture
```

## Compiled-binary gauntlet coverage

- `release_startup_navigation_gauntlet_proves_discovery_filters_and_help`
  covers startup logging, multi-session discovery, harness filtering, focus-aware footer states, help/legend provenance hints, theme cycling, and actionable focus/compose behavior in a temporary tmux socket.
- `release_action_gauntlet_proves_search_flash_sort_and_pane_operations`
  covers search confirm and cancel-restore, multiline compose, flash jump and jump+focus, sort cycling, rename, spawn, and kill flows through the compiled binary.
- `release_integration_gauntlet_proves_pr_notifications_and_graceful_degradation`
  covers PR lookup success and failure, browser/copy actions, notification backend fallback, mute/profile suppression, stable completion and attention transitions, and operator-visible failure logging.

## Supporting proof kept outside the gauntlet

- `tests/cli_config.rs` and `tests/bootstrap_observability.rs` cover config utility flow and startup log/bootstrap surfaces.
- `tests/claude_hook_bridge.rs`, `tests/codex_hook_bridge.rs`, and `tests/pi_hook_bridge.rs` cover native signal bridge correctness.
- `tests/claude_real_e2e.rs`, `tests/codex_real_e2e.rs`, and `tests/pi_real_e2e.rs` remain opt-in real-harness evidence for actual external CLIs.
- `tests/runtime_dashboard.rs` and `tests/runtime_profiling.rs` remain focused runtime/perf smokes that complement the broader release walkthrough.

## Notes

- This lane uses fake repos and fake notification/browser/clipboard helpers, but a real compiled `foreman` binary and a real tmux server.
- The gauntlet intentionally runs serially so pane focus, rename/spawn/kill, and notification timing remain deterministic.
