# Validation

## 2026-04-26

- `cargo test compatibility_status_maps_claude_phrases` — passed.
- `cargo test hook_error_signature_ignores_stale_scrollback_errors` — passed.
- `cargo test esc_quits_only_from_normal_mode` — passed.
- `cargo test run_prints_resolved_config_readout` — passed.
- `cargo test render_includes_main_surfaces` — passed.
- `cargo test --test runtime_dashboard` — passed, 10 tests.
- `cargo run -- --config-show` — passed and printed resolved config/UI/cache/runtime values for the current machine.
- `cargo run -- --bootstrap-only --debug` — passed; latest log no longer emitted the stale Claude hook-error operator alert after the fix.
- `cargo test --test release_gauntlet release_startup_navigation_gauntlet_proves_discovery_filters_and_help` — passed after updating the walkthrough for Esc-as-quit.
- `mise run check` — passed: format, lint, typecheck, tests, doc-tests.
