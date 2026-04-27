---
id: plan-validation
title: Validation Log
description: How changes were verified. Append entries after testing.
---

# Validation

- `cargo test --lib ui::render::tests -- --nocapture` — passed, 33 tests.
- `cargo test --test runtime_dashboard -- --nocapture` — passed, 10 tests.
- `cargo test --test release_gauntlet -- --test-threads=1 --nocapture` — passed, 3 tests.
- `mise run check` — passed.
- `mise run verify` — passed check + all-feature Rust integration/smoke phases; failed only at local Docker because Docker/Colima daemon is unavailable at `unix:///Users/alex.furrier/.colima/default/docker.sock`.
- `mise run verify-release` — passed.
- `mise run verify-ux` — passed and refreshed `.ai/validation/ux` screenshots/GIF.
## Review iteration revalidation

- `cargo test --lib ui::render::tests -- --nocapture` — passed, 33 tests after review fixes.
- `cargo test --test runtime_dashboard -- --nocapture` — passed, 10 tests after review fixes.
- `cargo test --test release_gauntlet -- --test-threads=1 --nocapture` — passed, 3 tests after review fixes.
- `mise run check` — passed after review fixes.
- `mise run verify-ux` — passed and refreshed UX screenshots/GIF after review fixes.
- `cargo test --test runtime_dashboard interactive_binary_surfaces_claude_native_status_and_attention_view -- --nocapture` — passed after tightening the Codex bot native-attention assertion.
## Native hook tmux e2e follow-up

- Manual tmux smoke: `danger` unsets `CLAUDECODE` but preserves `TMUX_PANE`; direct `foreman-claude-hook` writes `%pane.json` with `CLAUDECODE` unset.
- Manual live Claude smoke: fresh interactive `danger` pane falls back before first prompt; after submitting a prompt, Claude fires `UserPromptSubmit`, writes a native signal, and `foreman --doctor --claude-native-dir <temp>` reports native Claude active.
- `cargo test --test native_hook_tmux_e2e -- --nocapture` — passed, covering Claude, Codex, and Pi tmux-pane hook signal promotion.
- `cargo test --test claude_hook_bridge --test codex_hook_bridge --test pi_hook_bridge -- --nocapture` — passed.
- `mise run check` — passed with `native_hook_tmux_e2e` included in the full Rust test suite.
