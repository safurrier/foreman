---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for the operator-cockpit reframe and follow-up
  validation work.
---

# Specification — operator-cockpit-reframe

## Problem

Foreman is strong on native-vs-compatibility integration seams, test depth, and
tmux control basics. It is weaker on the part that should define the product:
fast triage across many parallel agents.

The current product shape is still tree-first:

- the main surface is a tmux session/window/pane tree
- attention exists mostly as a sort mode
- the detail panel takes a lot of space without carrying enough operator value
- compose is always visible even though it is not the default action most of the
  time
- spawn is a thin tmux window command, not a task or worktree flow

The next slice should not be a big-bang rewrite. It should preserve the good
parts of the current system:

- native mode stays more authoritative than compatibility mode
- pane ids stay canonical for tmux actions
- flash navigation, focus behavior, and keyboard-first operation stay fast
- tests keep proving real tmux behavior, not only unit behavior

The product goal is to move Foreman toward an attention-first cockpit:

- `Inbox` becomes the default landing view
- `All Agents` becomes the main browse/search view
- `Topology` stays available for exact tmux structure
- the current tree and detail surfaces are merged into a more useful context
  area instead of keeping both as separate heavyweight panels
- repo, worktree, branch, and PR state become first-class
- spawn becomes a task transaction with rollback and recovery
- lifecycle truth moves toward tmux events, process truth, and native signals,
  with text heuristics only as enrichment

This roadmap also needs to close the current operator-trust and release-proof
gaps that were found in review:

- notification suppression does not yet use actionable pane identity for
  session/window selections
- per-pane tmux capture failure is silent
- CI artifact upload paths are tied to dated `.ai/plans/...` directories
- PR CI does not exercise macOS before release time
- slow external lookups are still near the main loop and need a clearer path
  off it

## Requirements

### MUST

- Foreman keeps the current coarse operator-facing status model:
  - `working`
  - `needs attention`
  - `idle`
  - `error`
  - `unknown`
- Finer states such as `waiting_for_input`, `blocked`, `stopped`,
  `spawn_failed`, `pane_reused`, and `stale` are internal or secondary detail.
  They should feed `attention_reason`, lifecycle history, and notifications
  rather than replacing the top-level status model.
- `Inbox` becomes a first-class view separate from `Topology`.
- The current tmux tree is no longer the only primary navigation surface.
- The current detail panel is replaced by a compact context area that combines:
  - actionable target summary
  - repo/worktree/branch state
  - status source and confidence
  - recent output or transition history
- Compose is demoted to a drawer, modal, or collapsed region instead of taking
  permanent screen space in the default layout.
- `Topology` remains available as a secondary view for exact tmux structure and
  pane targeting.
- Native integrations stay more authoritative than compatibility heuristics.
- The roadmap introduces a repo/worktree service with first-class records for:
  - repo root
  - worktree path
  - branch
  - dirty state
  - ahead/behind state
  - PR lookup state
- Spawn becomes a transaction that can create a pane, window, or session, and
  can optionally create a new worktree and branch with rollback or recovery
  guidance on partial failure.
- Pane identity stays canonical for tmux actions, but the system also gains a
  separate task identity so pane reuse does not corrupt history, alerts, or
  spawn recovery.
- Notification suppression uses actionable pane identity, not only direct pane
  row selection.
- Per-pane `capture-pane` failure remains a soft failure, but it becomes visible
  in logs and operator-facing degraded-state language.
- CI artifact upload moves to stable output paths or a generated manifest and
  fails when expected proof is missing.
- Pull request CI adds a macOS lane with at least build plus a targeted smoke
  path before release time.
- The roadmap includes ADR and spec sync work, not only code changes.
- Every phase defines its dominant validation layer before implementation starts.

### SHOULD

- The new default layout should feel like “what needs me now?” first and “where
  is this in tmux?” second.
- The combined context area should reduce screen usage compared to the current
  detail panel while carrying more operator value.
- `All Agents` should support sort and filter by attention, activity, repo,
  branch, and harness.
- `Inbox` should contain only panes or tasks that need action or recent review.
- The first runtime step away from pure polling should be a hybrid design:
  periodic snapshots plus event reconciliation, not a risky full rewrite in one
  slice.
- Precedent notes, option matrices, and accepted tradeoffs should move into
  ADRs or design docs rather than staying inside the top-level product contract.

### Constraints

- Do not break the current native-hook bridges for Claude, Codex, and Pi while
  reshaping the product surface.
- Avoid a full runtime rewrite before the UI and task model are settled.
- Keep `render = pure(state)` and reducer/effect boundaries intact.
- Preserve keyboard-first behavior for focus, jump, spawn, kill, mute, and
  return paths.
- Validation must stay honest. New proof should land in stable artifact
  locations and avoid dated hard-coded paths.
- Prefer vertical slices that can land independently and keep `mise run check`
  and targeted heavy-lane proof green after each slice.
