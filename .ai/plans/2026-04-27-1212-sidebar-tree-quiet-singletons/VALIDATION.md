# Validation

Focused validation:

- `cargo test --lib app::state::tests -- --nocapture` — passed
- `cargo test --lib app::reducer::tests::move_selection_uses_sorted_visible_targets -- --nocapture` — passed
- `cargo test --lib app::reducer::tests::search_navigation_moves_between_filtered_matches -- --nocapture` — passed
- `cargo test --lib ui::render::tests::render_sidebar_keeps_tree_but_quiets_singleton_counts -- --nocapture` — passed

Full validation:

- `mise run check` — passed
- `mise run verify-ux` — passed; refreshed UX GIF/screenshots

