# Validation — startup-loading-flash-hint

## Planned validation

### Focused

- startup-loading unit tests
- runtime dashboard smoke for first paint before inventory fill
- flash/help render checks
- release gauntlet and profiling smokes affected by startup timing
- real Claude and Pi end-to-end lanes affected by selection timing

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
cargo test --lib render_shows_loading_state_before_first_inventory_refresh -- --nocapture
cargo test load_inventory_profiled_allows_startup_lazy_preview_capture -- --nocapture
cargo test --test runtime_dashboard interactive_binary_renders_loading_before_first_inventory_fill -- --nocapture
cargo test --lib render_shows_fixed_width_flash_labels -- --nocapture
cargo test --test runtime_dashboard -- --nocapture
cargo test --test notification_runtime -- --nocapture
cargo test --test release_gauntlet release_action_gauntlet_proves_search_flash_sort_and_pane_operations -- --nocapture
cargo test --test runtime_profiling navigation_burst_limits_pull_request_lookup_churn -- --ignored --nocapture
FOREMAN_REAL_CLAUDE_E2E=1 cargo test --test claude_real_e2e -- --ignored --nocapture
FOREMAN_REAL_PI_E2E=1 cargo test --test pi_real_e2e -- --ignored --nocapture
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

- `mise run verify` was confirmed end to end on this slice, including integration tests, Docker build, release gauntlet, and UX artifact refresh.
- Strict `verify-native` passed with real Claude, Codex, and Pi lanes enabled.
- During validation, release gauntlet and real Claude/Pi E2Es needed timing updates because the app now paints before inventory and selection settles on slightly different boundaries.
- Local Docker validation initially hit Colima space pressure, but the repo changes were fine. After cleanup, the Docker phase passed cleanly.
