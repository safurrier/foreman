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
