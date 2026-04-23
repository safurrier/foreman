---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

Planning-only inventory for this slice:

- Reviewed `SPEC.md` acceptance scenarios to anchor the operator-facing checklist.
- Reviewed current heavy validation entrypoints:
  - `.mise/tasks/verify`
  - `.mise/tasks/verify-ux`
- Reviewed current compiled-binary and tmux-backed proof in:
  - `tests/runtime_dashboard.rs`
  - `tests/runtime_profiling.rs`
  - `tests/tmux_inventory.rs`
  - `tests/tmux_focus.rs`
  - `tests/tmux_actions.rs`
  - `tests/tmux_flash.rs`
  - `tests/notification_runtime.rs`
  - `tests/pull_requests.rs`
  - `tests/claude_real_e2e.rs`
  - `tests/codex_real_e2e.rs`
  - `tests/pi_real_e2e.rs`

Commands used during planning:

```bash
gh pr checks 1
rg -n "^## A|^## R|^### A|^### R|Acceptance|must|MUST" SPEC.md
rg --files tests | sort
sed -n '392,680p' SPEC.md
sed -n '1,220p' .mise/tasks/verify-ux
sed -n '1,260p' tests/runtime_dashboard.rs
sed -n '1,260p' tests/runtime_profiling.rs
sed -n '1,240p' tests/support/tmux.rs
sed -n '1,220p' tests/tmux_actions.rs
sed -n '1,220p' tests/notification_runtime.rs
```

Implementation and final validation:

- Added reusable scenario helpers in `tests/support/release.rs` and extended
  `tests/support/tmux.rs` for better alternate-screen, focus, resize, and file
  assertions.
- Added `tests/release_gauntlet.rs` with three compiled-binary walkthroughs:
  startup/discovery, action flows, and PR/notification/degradation behavior.
- Added `.mise/tasks/verify-release` and wired `mise run verify` to include the
  release-confidence gauntlet and generated report artifact.
- Synced `README.md`, `SPEC.md`, `AGENTS.md`, `docs/architecture.md`, and
  `docs/workflows.md` to the new release-validation contract.

Artifacts:

- `.ai/plans/2026-04-10-151735-release-validation-gauntlet/artifacts/release-validation-report.md`
- `.ai/plans/2026-04-10-151735-release-validation-gauntlet/artifacts/release-gauntlet-output.txt`

Commands run during implementation and closeout:

```bash
cargo test --test release_gauntlet release_startup_navigation_gauntlet_proves_discovery_filters_and_help -- --exact --nocapture
cargo test --test release_gauntlet release_action_gauntlet_proves_search_flash_sort_and_pane_operations -- --exact --nocapture
cargo test --test release_gauntlet release_integration_gauntlet_proves_pr_notifications_and_graceful_degradation -- --exact --nocapture
mise run verify-release
mise run check
mise run verify
python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/verify_references.py .
python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/validate_frontmatter.py .
```

Final result:

- `cargo test --test release_gauntlet -- --test-threads=1 --nocapture` passed
  via `mise run verify-release`
- Native working-signal debounce now covers same-harness native snapshots as
  well as compatibility snapshots, which removed the CI-only brief-idle
  completion race in `release_integration_gauntlet`.
- `mise run check` passed
- `mise run verify` passed, including:
  - `cargo test --all-features`
  - Docker build
  - standalone release gauntlet
  - `verify-ux` runtime/profiling smoke and VHS artifact refresh
- Reference validation passed
- Frontmatter validation passed
