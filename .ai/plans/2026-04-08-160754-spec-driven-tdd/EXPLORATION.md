---
id: plan-exploration
title: Exploration Notes
description: >
  Current repo state, target completed-app sketch, and milestone ordering for
  the Foreman rebuild.
---

# Exploration - spec-driven-tdd

## Current codebase summary

- The Rust crate is still a scaffold: `src/main.rs` prints a placeholder
  message and `src/lib.rs` contains only example code.
- The durable product contract already exists in `SPEC.md`.
- The architecture doc exists and now records the target seams to build toward.
- `mise` tasks and plan scaffolding are already present, so the repo has a
  preferred validation and planning workflow.

## Target completed-app sketch

### Main surfaces

- **Header**: tmux health, agent counts by coarse status, active sort mode,
  notification mode, and local system pressure.
- **Sidebar**: tree of sessions, windows, and panes; collapse state; status
  markers; search results; flash labels when active.
- **Preview panel**: recent pane capture, selected pane metadata, integration
  source, and compact status explanation.
- **Input panel**: multiline compose box for sending text to the selected agent.
- **Footer**: visible mode, focus target, and contextual key hints.
- **Help surface**: command index and mode-specific behavior.

### Secondary surfaces

- Search overlay
- Flash navigation overlay
- Rename modal
- Spawn modal
- Confirm-kill modal
- Pull request detail panel

### Primary operator flows

1. Start Foreman and see all relevant agent work across tmux sessions.
2. Move quickly to the right pane with keyboard navigation, search, or flash
   labels.
3. Inspect pane output and decide whether the agent is working, done, blocked,
   or unclear.
4. Send text, rename, spawn, kill, or focus the selected pane.
5. Get notified only when the operator is not already looking at the relevant
   work.

## Milestone ordering

1. **Architecture spine**: commands, actions, reducer, state invariants, and
   render shell.
2. **Inventory loop**: tmux discovery, visibility rules, stable selection, and
   keyboard focus behavior.
3. **Status loop**: supported harness detection, coarse status model, and
   Claude native integration.
4. **Operator tools**: direct input, rename, spawn, kill, search, flash
   navigation, and sorting.
5. **Secondary surfaces**: pull request awareness, notifications, logging, and
   system stats.
6. **Acceptance sweep**: smoke tests, quality gates, and docs/spec sync.

## Critical files for the rebuild

- `SPEC.md`: source of truth for product behavior and acceptance scenarios
- `docs/architecture.md`: durable architecture and invariants
- `AGENTS.md`: repo workflow and pre-push contract
- `.mise/tasks/check`: fast quality gate
- `.mise/tasks/verify`: heavy validation gate
- `src/main.rs`: CLI entrypoint
- `src/lib.rs`: current scaffold to replace with real module structure
