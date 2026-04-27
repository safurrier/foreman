# Validation

Executed:

- `cargo test --lib app::state::tests -- --nocapture` — passed
- `cargo test --lib ui::render::tests::render_sidebar_elides_singleton_window_noise -- --nocapture` — passed
- `cargo test --test runtime_dashboard -- --nocapture` — passed
- `cargo test --test release_gauntlet -- --test-threads=1 --nocapture` — passed
- `mise run check` — passed after updating stale singleton-window navigation expectations
- `mise run verify-ux` — passed; refreshed `.ai/validation/ux/foreman-ux-diagnostic.gif` plus PNG snapshots

Findings:

- Full check caught reducer tests that still expected session -> window -> pane navigation for singleton agent windows.
- Notification runtime covered the same selection path; the compose step now moves session -> pane directly.
- UX profiling expected 9 visible targets for three singleton sessions; the expected count is now 6.
