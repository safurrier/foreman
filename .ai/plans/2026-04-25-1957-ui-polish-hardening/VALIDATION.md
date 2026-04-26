# Validation

## 2026-04-25

Focused regression tests passed:

- `cargo test ui_preferences_round_trip`
- `cargo test reset_ui_preferences_removes_existing_file_and_tolerates_missing_file`
- `cargo test explicit_config_ui_keys_win_over_persisted_preferences`
- `cargo test reset_ui_state_removes_persisted_preferences_and_exits`
- `cargo test config_findings_report_invalid_ui_preferences`
- `cargo test config_findings_report_readable_ui_preferences`
- `cargo test stale_startup_cache_keeps_inventory_counts_for_diagnostics`
- `cargo test config_findings_report_startup_cache_inventory_readout`
- `cargo test completion_transition_emits_notification_when_unsuppressed`
- `cargo test render_shows_pull_request_refresh_feedback`
- `cargo test advertised_normal_mode_keybinds_map_to_expected_commands`
- `cargo test preview_scroll_mode_keeps_normal_shortcuts_available`
- `cargo test --test runtime_dashboard`

Full gate passed:

- `mise run check` — passed: format, lint, typecheck, tests, doc-tests.
