---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
---

# Validation

Planning inventory:

- Reviewed `src/app/state.rs`, `src/app/reducer.rs`, `src/app/command.rs`,
  `src/ui/render.rs`, `.mise/tasks/verify`, `.mise/tasks/verify-release`, and
  `tests/release_gauntlet.rs`.
- Confirmed the current gaps:
  - harness cycling still includes empty views
  - help text still says `f tmux focus` instead of spelling out the behavior
  - keybind proof is broad but not an explicit matrix
  - `mise run verify` intentionally reruns the release gauntlet after `cargo
    test --all-features`; that redundancy should be kept but documented

Implementation validation will be appended below.

Implementation and closeout:

- Added `Inventory::available_harnesses()` and changed harness cycling to use
  only harnesses present in the current inventory before returning to `all`.
- Tightened operator copy in `src/ui/render.rs` so the preview/help/input/footer
  spell out that `f` jumps tmux to the resolved target pane.
- Added explicit advertised-key tests in `src/app/command.rs`.
- Strengthened live tmux proof in:
  - `tests/runtime_dashboard.rs`
  - `tests/release_gauntlet.rs`
- Added `.mise/tasks/verify-native` and synced README/spec/workflow docs.
- Updated `.mise/tasks/verify-ux` to restore a stable placeholder
  `visual-env.txt` after capture so the verification lane does not leave behind
  random temp-path churn.

Artifacts touched:

- `.ai/plans/2026-04-10-110931-tui-ux-diagnostic/artifacts/foreman-ux-diagnostic.gif`
- `.ai/plans/2026-04-10-110931-tui-ux-diagnostic/artifacts/foreman-ux-initial.png`
- `.ai/plans/2026-04-10-110931-tui-ux-diagnostic/artifacts/foreman-ux-gruvbox.png`
- `.ai/plans/2026-04-10-110931-tui-ux-diagnostic/artifacts/foreman-ux-no-color.png`
- `.ai/plans/2026-04-10-110931-tui-ux-diagnostic/artifacts/foreman-ux-help.png`
- `.ai/plans/2026-04-10-110931-tui-ux-diagnostic/artifacts/foreman-ux-search.png`
- `.ai/plans/2026-04-10-110931-tui-ux-diagnostic/artifacts/foreman-ux-flash.png`
- `.ai/plans/2026-04-10-151735-release-validation-gauntlet/artifacts/release-validation-report.md`
- `.ai/plans/2026-04-10-151735-release-validation-gauntlet/artifacts/release-gauntlet-output.txt`

Commands run:

```bash
cargo fmt
cargo test --lib
cargo test --test runtime_dashboard -- --nocapture
cargo test --test release_gauntlet release_startup_navigation_gauntlet_proves_discovery_filters_and_help -- --exact --nocapture
mise run verify-native
mise run verify-release
mise run verify-ux
mise run check
python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/verify_references.py .
python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/validate_frontmatter.py .
mise run verify
```

Final result:

- `cargo test --lib` passed
- `cargo test --test runtime_dashboard -- --nocapture` passed
- `cargo test --test release_gauntlet release_startup_navigation_gauntlet_proves_discovery_filters_and_help -- --exact --nocapture` passed
- `mise run verify-native` passed with the expected skip output when no real-harness env flags were set
- `mise run verify-release` passed
- `mise run verify-ux` passed and refreshed the tracked UX artifacts
- `mise run check` passed
- reference validation passed
- frontmatter validation passed
- `mise run verify` passed, including:
  - `cargo test --all-features`
  - Docker build
  - the intentionally repeated `verify-release` run
  - `verify-ux` capture refresh
- After the `verify-ux` placeholder cleanup landed, `mise run verify` passed a
  second time from the final tree state.
