---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step roadmap for the operator-cockpit reframe and validation follow-up
  work.
---

# Implementation — operator-cockpit-reframe

## Guiding Shape

Keep the current technical strengths, but change the product center of gravity.

Target layout:

1. Left rail: view switcher plus list surface
   - `Inbox`
   - `All Agents`
   - `Topology`
2. Main area: combined context surface
   - target card
   - repo/worktree card
   - recent output or transition history
3. Compose: collapsed by default, opened on demand

This keeps the tree when it is useful, but it stops making the tmux tree the
default product thesis.

## ADR Plan

Write these before or with the matching implementation slices:

1. `0002-operator-cockpit-information-architecture.md`
   - Why `Inbox` becomes primary
   - Why `Topology` becomes secondary
   - Why detail and tree are merged into a context surface
2. `0003-lifecycle-truth-hierarchy.md`
   - Snapshot polling vs tmux events vs native hooks vs heuristics
   - Why pane id is canonical for tmux actions
   - Why task identity must exist separately from pane identity
3. `0004-spawn-transaction-and-worktree-model.md`
   - Why spawn becomes transactional
   - Rollback and recovery rules
   - Repo/worktree record boundaries

## Phases

### Phase 0: Contract and direction lock

Goal:
- Freeze the product direction before touching broad UI or runtime code.

Work:
- Update `SPEC.md` to describe the attention-first product shape.
- Move precedent notes, alternatives, and accepted tradeoffs into ADRs.
- Update `docs/architecture.md` with the planned repo/worktree, task identity,
  and lifecycle-engine seams.
- Update `README.md` and `docs/workflows.md` so the roadmap and validation lanes
  stay aligned.

Validation:
- reference checks
- frontmatter checks
- doc review for spec/architecture consistency

### Phase 1: Trust fixes and validation hardening

Goal:
- Close the current operator-trust and release-proof gaps without waiting for
  the larger reframe.

Work:
- Fix notification suppression to use actionable pane identity for pane,
  window, and session selections.
- Surface per-pane capture failure as first-class observability:
  - log pane id and error
  - expose degraded status/source copy in UI
  - keep soft-failure behavior
- Replace dated CI/release artifact upload paths with stable paths or a
  generated manifest.
- Add a macOS PR lane with build plus a targeted smoke path.
- Tighten runtime profiling and assertions around slow external lookup work.

Validation:
- reducer tests for actionable-notification suppression
- adapter contract tests for capture failure observability
- actionlint
- targeted GitHub workflow tests where possible
- `cargo test --lib`
- `mise run check`

### Phase 2: UI reshape without runtime rewrite

Goal:
- Change the default operator experience before deeper runtime work.

Work:
- Add first-class `Inbox`, `All Agents`, and `Topology` views.
- Keep `Topology` backed by the current tree model.
- Replace the current heavyweight detail panel with a combined context surface.
- Demote compose to a drawer or modal path.
- Keep flash navigation and focus behavior fast across all views.
- Introduce finer internal attention reasons while keeping the top-level status
  model coarse.

Validation:
- render tests for each view and layout breakpoint
- tmux-backed runtime smoke for navigation, jump, and compose drawer behavior
- UX artifact refresh
- release gauntlet updates for the new default flow

### Phase 3: Repo, worktree, and spawn transaction model

Goal:
- Make Foreman good at starting and tracking isolated tasks, not only watching
  existing panes.

Work:
- Introduce repo/worktree records in state and services.
- Add branch, dirty, ahead/behind, and cached PR state as repo facets.
- Replace raw `SpawnWindow { command }` with a spawn transaction:
  - resolve repo root
  - optionally create worktree and branch
  - create pane/window/session
  - start command template
  - record step journal
  - rollback or expose recovery path on failure
- Update the combined context area to show repo and worktree state inline.

Validation:
- unit tests for repo-state parsing and transaction journaling
- fake-backed contract tests for git and PR services
- tmux integration tests for successful spawn and step-failure recovery
- targeted release-gauntlet scenario for worktree spawn

### Phase 4: Lifecycle engine and task identity

Goal:
- Move from “poll and infer” toward “reconcile snapshots, events, and native
  signals.”

Work:
- Add tmux event ingestion or control-mode reconciliation as a runtime seam.
- Reconcile event truth with periodic snapshot truth.
- Add remain-on-exit and pane-reuse handling.
- Introduce task identity separate from pane identity.
- Keep native signals as the most semantic source where available.
- Restrict text heuristics to enrichment and fallback.

Validation:
- unit tests for lifecycle classifier and task reassignment rules
- integration tests with ephemeral tmux worlds for exit, remain-on-exit, pane
  reuse, and external kill cases
- performance runs at 20 and 50 panes
- runtime profiling to prove lookup work is off the hot path

### Phase 5: Follow-through and cleanup

Goal:
- Make the new shape durable and review-friendly.

Work:
- Clean old tree/detail assumptions out of copy and docs.
- Add durable examples for the new spawn and triage flows.
- Re-check CI artifact paths, review evidence surfacing, and release drills.
- Revisit future-work items such as custom sound profiles only after the core
  cockpit shape is stable.

Validation:
- `mise run verify-ux`
- `mise run verify-release`
- `mise run verify`
- final spec/doc/ADR sync review

## Slice Order

Recommended landing order:

1. Phase 0
2. Phase 1
3. Phase 2
4. Phase 3
5. Phase 4
6. Phase 5

This order gives earlier operator value and lowers risk:

- fix current trust gaps first
- prove the product shape in the UI before deeper runtime changes
- add repo/worktree spawn value before the event-engine rewrite
- move lifecycle truth last, once the task model is clear
