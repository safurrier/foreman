# Validation

## 2026-04-24

- `cargo test config_parsing_supports_public_default_sort_names` — passed.
- `cargo test config_parsing_supports_notification_and_integration_preferences` — passed.
- `cargo test runtime_config_applies_cli_overrides` — passed.
- `cargo test runtime_bootstrap_uses_fresh_popup_startup_cache` — passed.
- `cargo test bootstrap_log_writes_runtime_summary` — passed.
- `cargo test ui_preferences_round_trip` — passed.
- `cargo test render_surfaces_cached_startup_provenance` — passed.
- `cargo test render_explains_pull_request_unavailable_reason` — passed.
- `cargo test notification` — passed.
- `cargo test config_findings_report_startup_cache_inventory_readout` — passed.
- `cargo test --test runtime_dashboard` — passed after fixing UI state path isolation.
- `mise run check` — passed: format, lint, typecheck, tests, doc-tests.
