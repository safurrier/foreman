# Validation — capture-refresh-perf

## Planned validation

### Focused

- config/default regression tests
- tmux adapter unit tests for prioritized capture
- runtime dashboard smoke for preview freshness on selected and visible rows
- runtime profiling smoke for reduced `inventory_refresh` cost

### Full

```bash
mise run check
mise run verify
mise run native-preflight
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

## Status

Validation completed on the final tree.

### Focused runs

```bash
cargo test load_inventory_profiled_reuses_stable_offscreen_previews -- --nocapture
cargo test refresh_priority_pane_ids_include_viewport_selected_and_attention_rows -- --nocapture
cargo test --test runtime_dashboard -- --nocapture
cargo test bootstrap_log_writes_runtime_summary -- --nocapture
cargo test --test runtime_profiling -- --ignored --nocapture
cargo test --test release_gauntlet release_integration_gauntlet_proves_pr_notifications_and_graceful_degradation -- --nocapture
cargo test --test notification_runtime -- --nocapture
```

Result: all passed.

### Full validation

```bash
mise run check
mise run verify
mise run native-preflight
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

Result: all passed.

### Notes

- `mise run verify` passed with the crowded profiling smokes, release gauntlet, Docker build, and UX artifact refresh included.
- Strict `verify-native` passed with real Claude, Codex, and Pi end-to-end lanes enabled.
- During closeout, `tests/notification_runtime.rs` was hardened to wait on the expected log signal rather than a fixed sleep so the fast gate would remain deterministic after async refresh work.
