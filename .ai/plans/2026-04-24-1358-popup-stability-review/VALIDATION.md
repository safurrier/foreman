# Validation

Validation will be appended as commands are run.

## 2026-04-24 14:04

- `cargo test recent_sort_stays_stable_across_status_changes` — passed.
- `cargo test popup_mode_does_not_suppress_selected_pane_notifications` — passed.
- `cargo test render_surfaces_compound_sort_labels` — passed.
- `cargo test render_footer_uses_labeled_control_groups` — passed.
- `cargo test --test notification_runtime runtime_uses_configured_notification_backend_order` — passed after selecting `alpha` by search rather than fixed row count.
- `cargo test search_commit_selects_matching_session_header` — passed.
- `cargo test flash_jump_and_focus_resolves_session_to_actionable_pane` — passed.
- `cargo test --test runtime_dashboard interactive_binary_popup_focus_action_switches_cross_session_target -- --nocapture` — passed after waiting for live `beta` inventory before searching.
- `cargo fmt` — applied required formatting.
- `mise run check` — passed: format, lint, typecheck, tests, doc-tests.
