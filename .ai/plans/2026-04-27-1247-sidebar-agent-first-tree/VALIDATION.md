# Validation

Focused validation:

- `cargo test --lib app::state::tests -- --nocapture` — passed
- `cargo test --lib app::reducer::tests::move_selection_uses_sorted_visible_targets -- --nocapture` — passed
- `cargo test --lib app::reducer::tests::search_navigation_moves_between_filtered_matches -- --nocapture` — passed
- `cargo test --lib ui::render::tests::render_sidebar_shows_agent_first_tree_without_singleton_window_noise -- --nocapture` — passed
- `cargo test --lib ui::render::tests::render_popup_prefers_side_by_side_tree_layout_at_typical_size -- --nocapture` — passed

Full validation:

- `mise run check` — passed
- `mise run verify-ux` — passed; refreshed regular and popup UX artifacts

