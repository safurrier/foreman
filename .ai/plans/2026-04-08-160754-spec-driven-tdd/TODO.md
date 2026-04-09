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
- [ ] Implement `foreman --config-path`
- [ ] Implement `foreman --init-config`
- [ ] Parse runtime overrides for polling, capture depth, popup mode, and notifications
- [ ] Add boot logging and latest-log pointer behavior

## Phase 2: Render shell
- [ ] Render header, sidebar, preview, input, footer, and help shell from pure state
- [ ] Add buffer tests for focus styling, mode visibility, and empty-state error shell
- [ ] Ensure the shell is usable when tmux is missing or empty

## Phase 3: Inventory and visibility
- [ ] Add tmux discovery adapter with fake-backed contract tests
- [ ] Populate multi-session inventory across sessions and windows
- [ ] Hide non-agent sessions and panes by default, with runtime toggles
- [ ] Preserve collapse state and logical selection across refreshes

## Phase 4: Navigation and focus
- [ ] Implement keyboard navigation through visible targets
- [ ] Implement panel focus changes and escape semantics
- [ ] Implement tmux focus action
- [ ] Close automatically after successful focus in popup mode

## Phase 5: Agent detection and status
- [ ] Add compatibility-mode harness detection for supported families
- [ ] Implement coarse status model: working, needs attention, idle, error, unknown
- [ ] Add status debouncing to prevent flicker
- [ ] Add Claude Code native integration and native-over-compatibility precedence

## Phase 6: Direct actions
- [ ] Implement multiline direct input to selected pane
- [ ] Implement confirmed kill action
- [ ] Implement window rename action
- [ ] Implement spawn-new-agent action in selected session

## Phase 7: Discovery tools
- [ ] Implement interactive search with cancel-and-restore semantics
- [ ] Implement flash navigation with overflow labels
- [ ] Implement sort modes: recent activity and attention-first
- [ ] Preserve logical selection across sort changes where possible

## Phase 8: Secondary surfaces
- [ ] Implement pull request awareness with graceful degradation
- [ ] Implement notification suppression, cooldowns, mute, and profile switching
- [ ] Surface operator-visible failures in the UI and logs
- [ ] Add header stats for local CPU and memory pressure

## Phase 9: Acceptance and docs
- [ ] Add shell-based smoke tests for core operator flows
- [ ] Run `cargo fmt`, `cargo clippy`, `cargo test`, and `mise run check`
- [ ] Sync runtime behavior back into `README.md`, `SPEC.md`, and `docs/architecture.md` as needed
