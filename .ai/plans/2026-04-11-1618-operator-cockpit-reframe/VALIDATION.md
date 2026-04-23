---
id: plan-validation
title: Validation Log
description: >
  Planning inventory and validation strategy for the operator-cockpit reframe.
---

# Validation

Planning inventory:

- Reviewed the current product contract in:
  - `SPEC.md`
  - `docs/architecture.md`
  - `docs/workflows.md`
- Reviewed the current runtime and adapter seams in:
  - `src/runtime.rs`
  - `src/adapters/tmux.rs`
  - `src/integrations/*.rs`
  - `src/services/notifications.rs`
  - `src/services/pull_requests.rs`
- Reviewed current state and UI shape in:
  - `src/app/state.rs`
  - `src/app/action.rs`
  - `src/app/reducer.rs`
  - `src/ui/render.rs`
- Reviewed live proof and release confidence in:
  - `tests/runtime_dashboard.rs`
  - `tests/tmux_flash.rs`
  - `tests/notification_runtime.rs`
  - `tests/release_gauntlet.rs`

Confirmed gaps this roadmap addresses:

- The current product is still tree-first.
- Attention is a sort mode, not a first-class view.
- Compose takes permanent space in the default layout.
- Spawn is still a thin tmux window command.
- Repo and worktree state are not first-class.
- Notification suppression misses actionable session/window selections.
- Per-pane capture failure is silent.
- CI artifact upload paths are brittle.
- PR CI does not cover macOS early enough.
- Runtime already knows slow external lookups are a risk and should move farther
  off the hot path.

Planned validation strategy by phase:

- Phase 0:
  - spec and ADR review
  - reference and frontmatter validation
- Phase 1:
  - reducer and adapter tests
  - actionlint
  - targeted workflow proof
  - `mise run check`
- Phase 2:
  - render tests
  - runtime smoke
  - UX artifact refresh
  - release gauntlet updates
- Phase 3:
  - unit tests for repo and spawn transactions
  - fake-backed contract tests
  - tmux integration and failure-injection tests
- Phase 4:
  - lifecycle integration tests
  - perf and runtime profiling at load
  - negative tests for exit, reuse, and stale state
- Phase 5:
  - full heavy-lane re-run
  - final spec/doc/ADR sync review

Implementation validation will be appended here as slices land.
