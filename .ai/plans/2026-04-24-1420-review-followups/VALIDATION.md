# Validation

## 2026-04-24

- `cargo test failed_input_send_restores_compose_draft` — passed.
- `cargo test failed_rename_restores_modal_draft` — passed.
- `cargo test failed_spawn_restores_modal_draft` — passed.
- `cargo test command_wait_times_out_slow_processes` — passed.
- `cargo test service_delegates_lookup_and_auxiliary_actions` — passed.
- `cargo test preview_focus_maps_navigation_to_preview_scroll` — passed.
- `cargo test preview_scroll_actions_are_reducer_owned_and_reset_on_selection_change` — passed.
- `cargo test --test runtime_dashboard interactive_binary_footer_tracks_focus_and_help_explains_provenance` — passed.
- `mise run check` — passed: format, lint, typecheck, tests, doc-tests.

## 2026-04-24 PR Review Loop

- Codex self-review identified destructive stale native signal pruning as unsafe across tmux contexts; removed pruning instead of weakening isolation.
- Added preview-scroll clamp coverage for oversized scroll offsets.
- Updated plan metadata with PR URL and in-progress lifecycle status.

### Re-run After PR Review Fixes

- `cargo test render_preview_clamps_large_scroll_offsets` — passed.
- `cargo test stable_sort_stays_stable_across_status_changes` — passed.
- `cargo test returns_none_for_editor_panes_with_stale_harness_breadcrumbs` — passed.
- `cargo test compatibility_snapshot_drops_editor_panes_with_stale_harness_text` — passed.
- `cargo test default_visibility_hides_non_agent_sessions_and_panes` — passed.
- `mise run check` — passed: format, lint, typecheck, tests, doc-tests.

### External PR Comment Fixes

- `cargo test preview_scroll_mode_keeps_normal_shortcuts_available` — passed.
- `cargo test search_commit_resets_preview_scroll` — passed.
- `mise run check` — passed after external PR comment fixes.
