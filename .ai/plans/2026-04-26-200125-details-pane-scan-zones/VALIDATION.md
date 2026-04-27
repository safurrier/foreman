---
id: plan-validation
title: Validation Log
description: How changes were verified. Append entries after testing.
---

# Validation
## Local validation

- `cargo test --lib ui::render::tests -- --nocapture` — passed, 33 tests.
- `cargo test --test runtime_dashboard -- --nocapture` — passed, 10 tests.
- `cargo test --test release_gauntlet -- --test-threads=1 --nocapture` — passed, 3 tests.
- `mise run check` — passed.
- `mise run verify-ux` — passed and refreshed `.ai/validation/ux` screenshots/GIF.
- Manual screenshot review: `.ai/validation/ux/foreman-ux-initial.png` shows Details rows with stable labels and no-color-compatible separators.
## Self-review iteration

- Codex self-review found two medium issues: max-width detail-label spacing and overly broad runtime/release waits.
- Fixed spacing so `Preview source` keeps a visible value gap and added render assertions against `Preview source:captured`.
- Tightened runtime/release waits to assert Details scan rows such as `Target:         ✦ alpha` and `Status source:  native hook`.
- `cargo test --lib ui::render::tests -- --nocapture` — passed, 33 tests.
- `cargo test --test runtime_dashboard -- --nocapture` — passed, 10 tests.
- `cargo test --test release_gauntlet -- --test-threads=1 --nocapture` — passed, 3 tests.
- `mise run check` — passed after review fixes.
- `mise run verify-ux` — passed after review fixes and refreshed screenshots/GIF.
