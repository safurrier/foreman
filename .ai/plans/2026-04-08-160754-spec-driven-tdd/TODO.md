---
id: plan-todo
title: Task List
description: >
  Checkable tasks for this unit of work. Check off as you go.
---

# TODO - spec-driven-tdd

## Phase -1: Product sketch and traceability
- [x] Sketch the target completed app and primary operator flows
- [x] Define milestone ordering before implementation starts
- [x] Create a SPEC traceability and architecture review checklist
- [x] Add detailed acceptance and validation specs for the remaining chunks

## Phase 0: Architecture spine
- [x] Define core app vocabulary: `Command`, `Action`, `Mode`, `Focus`, `SortMode`, `AgentStatus`, `IntegrationMode`
- [x] Add pure reducer tests for mode precedence, selection stability, and filter persistence
- [x] Add fixture builders for sessions, windows, panes, and agent snapshots

## Phase 1: CLI, config, and boot
- [x] Implement `foreman --config-path`
- [x] Implement `foreman --init-config`
- [x] Parse runtime overrides for polling, capture depth, popup mode, and notifications
- [x] Add boot logging and latest-log pointer behavior

## Phase 2: Render shell
- [x] Render header, sidebar, preview, input, footer, and help shell from pure state
- [x] Add buffer tests for focus styling, mode visibility, and empty-state error shell
- [x] Ensure the shell is usable when tmux is missing or empty

## Phase 3: Inventory and visibility
- [x] Add tmux discovery adapter with fake-backed contract tests
- [x] Populate multi-session inventory across sessions and windows
- [x] Hide non-agent sessions and panes by default, with runtime toggles
- [x] Preserve collapse state and logical selection across refreshes
- [x] Add real tmux fixture smoke / E2E coverage for startup inventory and visibility rules

## Phase 4: Navigation and focus
- [x] Implement keyboard navigation through visible targets
- [x] Implement panel focus changes and escape semantics
- [x] Implement tmux focus action
- [x] Close automatically after successful focus in popup mode
- [x] Add real tmux focus coverage and popup-close effect assertions

## Phase 5: Agent detection and status
- [x] Add compatibility-mode harness detection for supported families
- [x] Implement coarse status model: working, needs attention, idle, error, unknown
- [x] Add status debouncing to prevent flicker
- [x] Add Claude Code native integration and native-over-compatibility precedence
- [x] Add real tmux + native-shim coverage for precedence and fallback behavior

## Phase 6: Direct actions
- [x] Implement multiline direct input to selected pane
- [x] Implement confirmed kill action
- [x] Implement window rename action
- [x] Implement spawn-new-agent action in selected session
- [x] Add real tmux action coverage for send, rename, spawn, and kill

## Phase 7: Discovery tools
- [x] Implement interactive search with cancel-and-restore semantics
- [x] Implement flash navigation with overflow labels
- [x] Implement sort modes: recent activity and attention-first
- [x] Preserve logical selection across sort changes where possible
- [x] Add focused real tmux coverage for flash jump-and-focus behavior

## Phase 8: Secondary surfaces
- [x] Implement pull request awareness with graceful degradation
- [ ] Implement notification suppression, cooldowns, mute, and profile switching
- [ ] Surface operator-visible failures in the UI and logs
- [ ] Add header stats for local CPU and memory pressure

## Phase 9: Acceptance and docs
- [ ] Add shell-based smoke tests for core operator flows
- [ ] Run `cargo fmt`, `cargo clippy`, `cargo test`, and `mise run check`
- [ ] Sync runtime behavior back into `README.md`, `SPEC.md`, and `docs/architecture.md` as needed
