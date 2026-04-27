# Validation

Focused validation:

- `cargo test --lib app::state::tests::agent_first_sidebar_uses_elided_window_title_for_direct_pane_rows -- --nocapture` — passed
- `cargo test --lib ui::render::tests::render_displays_inline_search_footer_and_match_count -- --nocapture` — passed
- `cargo test --lib ui::render::tests::render_footer_uses_labeled_control_groups -- --nocapture` — passed
- `cargo test --test runtime_dashboard interactive_binary_footer_tracks_focus_and_help_explains_provenance -- --nocapture` — passed

Full validation:

- `mise run check` — passed
- `mise run verify-ux` — passed; refreshed regular/search/popup UX artifacts

