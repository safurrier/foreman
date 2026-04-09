---
id: plan-chunk-specs
title: Remaining Chunk Specs
description: >
  Detailed acceptance, architecture, and validation gates for the remaining
  implementation chunks after the state core.
---

# Remaining Chunk Specs - spec-driven-tdd

## How to use this file

Before starting any chunk:

1. Confirm the matching checklist in `TRACEABILITY.md`
2. Name the dominant test layer before writing code
3. Keep the implementation inside the planned file seams unless the chunk forces
   an architectural update
4. Update `SPEC.md` or `docs/architecture.md` if the chunk changes a contract or
   invariant
5. Do not begin the next chunk until the current chunk's validation gate is green

Chunk 1 is already complete in commit `fcbef09`.

## Chunk 2 - CLI and config utility flow

**Goal**
Turn `main.rs` into a real CLI entrypoint with typed config/bootstrap behavior.

**SPEC coverage**
- R1-R3
- A1-A2

**Acceptance checklist**
- [ ] `foreman --config-path` prints the expected config location and exits 0
- [ ] `foreman --init-config` creates a default config and exits 0
- [ ] Runtime overrides exist for polling, capture depth, popup mode, and notification disablement
- [ ] Normal interactive boot initializes logging and latest-run pointer behavior

**Architecture guardrails**
- Keep CLI parsing in `src/cli.rs`
- Keep config types separate from `AppState`
- Logging bootstrap belongs in a service seam, not in ad hoc `main.rs` logic

**Dominant validation**
- Unit tests for CLI parsing and config path/init behavior
- Shell smoke tests for `--config-path` and `--init-config`
- `mise run check`

**Likely files**
- `src/main.rs`
- `src/cli.rs`
- `src/config.rs`
- `src/services/logging.rs`
- tests for CLI/config behavior

## Chunk 3 - Render shell and empty-state dashboard

**Goal**
Render the stable dashboard shell from pure state before wiring live tmux data.

**SPEC coverage**
- R9
- tmux empty-shell contract

**Acceptance checklist**
- [ ] Header, sidebar, preview, input, footer, and help surfaces render from `AppState`
- [ ] Focused panel is visually obvious
- [ ] Mode is always visible in the footer or status line
- [ ] Missing tmux shows a visible empty/error shell instead of crashing

**Architecture guardrails**
- `render = pure(state)` remains true
- No tmux calls inside render code
- UI should support monochrome-safe status presentation from the start

**Dominant validation**
- Ratatui buffer tests
- Focus and mode visibility assertions
- `mise run check`

**Likely files**
- `src/ui/render.rs`
- `src/ui/layout.rs`
- `src/ui/widgets/*`
- `src/app/state.rs`

## Chunk 4 - tmux inventory and visibility rules

**Goal**
Add a tmux adapter seam and populate the sidebar inventory with real session,
window, and pane data.

**SPEC coverage**
- R4
- R7
- A3
- A6

**Acceptance checklist**
- [ ] Sessions across tmux are discovered, not just the attached session
- [ ] Pane capture is available as recent preview input
- [ ] Non-agent sessions and panes are hidden by default
- [ ] Runtime toggles reveal hidden sessions and panes
- [ ] Collapse state and logical selection survive refresh

**Architecture guardrails**
- All tmux I/O lives behind `src/adapters/tmux.rs`
- Reducer receives normalized inventory data, not raw command output
- Refresh cadence should be configuration-driven

**Dominant validation**
- Adapter contract tests with a fake tmux backend
- Reducer tests for visibility and reconciliation behavior
- Focused smoke test against a real tmux fixture

**Likely files**
- `src/adapters/tmux.rs`
- `src/app/action.rs`
- `src/app/reducer.rs`
- `src/app/state.rs`
- integration-style tmux fixture tests

## Chunk 5 - Keyboard navigation and focus actions

**Goal**
Make the dashboard feel like a real keyboard-first operator console.

**SPEC coverage**
- R10-R11
- A8-A9

**Acceptance checklist**
- [ ] Keyboard navigation advances through visible targets
- [ ] Focus switches cleanly between sidebar, preview, and input
- [ ] Escape semantics match the active mode
- [ ] Focusing a pane switches tmux context as needed
- [ ] Popup mode closes automatically after successful focus

**Architecture guardrails**
- Raw key handling maps to `Command`, then `Action`
- Keyboard navigation stays instant with no animation delay
- Popup auto-exit is an effect after successful tmux focus, not a render concern

**Dominant validation**
- Command mapping unit tests
- Ratatui buffer tests for visible focus state
- tmux adapter contract tests for focus behavior
- Popup smoke test

**Likely files**
- `src/app/command.rs`
- `src/app/action.rs`
- `src/app/reducer.rs`
- `src/ui/render.rs`
- `src/adapters/tmux.rs`

## Chunk 6 - Compatibility detection and coarse status

**Goal**
Recognize supported harness panes in compatibility mode and derive the first
operator-facing status model.

**SPEC coverage**
- R5-R8
- A4
- A7

**Acceptance checklist**
- [ ] Claude Code, Codex CLI, Gemini CLI, and OpenCode panes can be recognized in compatibility mode
- [ ] Each recognized pane resolves to one of: working, needs attention, idle, error, unknown
- [ ] Brief signal loss does not flicker status immediately
- [ ] Unrecognized panes remain present when visibility allows

**Architecture guardrails**
- Detection logic stays separate from tmux transport code
- Status derivation is pure and unit-testable from normalized observations
- Debounce policy is centralized, not scattered across integrations

**Dominant validation**
- Unit tests for detection and status mapping
- Unit tests for debounce behavior
- Fake capture contract tests

**Likely files**
- `src/integrations/mod.rs`
- `src/integrations/claude.rs`
- `src/integrations/codex.rs`
- `src/integrations/gemini.rs`
- `src/integrations/opencode.rs`
- `src/app/state.rs`

## Chunk 7 - Claude native integration and precedence

**Goal**
Add the first native integration and make native signals authoritative.

**SPEC coverage**
- R5-R6
- A5
- A11
- integration precedence invariant

**Acceptance checklist**
- [ ] Claude Code can operate with a native signal source
- [ ] Native state wins when both native and compatibility signals are available
- [ ] Foreman falls back to compatibility mode if the native path disappears
- [ ] Claude completion and attention signals are stable enough to drive notifications

**Architecture guardrails**
- Precedence is enforced in one place
- Native and compatibility adapters share the same normalized status shape
- Fallback behavior is explicit and logged

**Dominant validation**
- Unit tests for precedence and fallback
- Native-signal adapter contract tests
- Focused smoke test with a Claude fixture or shim

**Likely files**
- `src/integrations/claude.rs`
- `src/app/reducer.rs`
- `src/app/state.rs`
- `src/services/logging.rs`

## Chunk 8 - Direct input and operational actions

**Goal**
Let the operator intervene directly in the selected pane.

**SPEC coverage**
- R12-R13
- A10
- A12

**Acceptance checklist**
- [ ] Multiline input composes and sends to the selected pane
- [ ] Non-ASCII text handling remains safe
- [ ] Kill requires explicit confirmation
- [ ] Window rename updates the selected window
- [ ] Spawn creates a new agent window in the selected session

**Architecture guardrails**
- Text editing state belongs in app state, not ad hoc widget state
- Destructive actions must pass through explicit modal state
- tmux action adapters return structured success/failure results

**Dominant validation**
- Reducer tests for input and modal state transitions
- Adapter contract tests for send, kill, rename, and spawn
- Shell smoke tests for real pane operations

**Likely files**
- `src/app/state.rs`
- `src/app/action.rs`
- `src/app/reducer.rs`
- `src/ui/widgets/input.rs`
- `src/adapters/tmux.rs`

## Chunk 9 - Search, flash navigation, and sorting

**Goal**
Add fast discovery tools without destabilizing selection behavior.

**SPEC coverage**
- R14-R16
- A13-A15

**Acceptance checklist**
- [ ] Search filters visible items by query
- [ ] Search can move between matches
- [ ] Cancel restores the previous cursor position
- [ ] Flash navigation supports jump-only and jump-and-focus flows
- [ ] Overflow labels work beyond single-character labels
- [ ] Sort mode changes preserve logical selection where possible

**Architecture guardrails**
- Search and flash state remain high-priority modal state, not scattered widget state
- Sort changes always reconcile selection by logical identity
- Flash label assignment is deterministic for visible targets

**Dominant validation**
- Reducer tests for search, flash, and sort semantics
- Ratatui buffer tests for overlays
- Focused smoke tests for jump-and-focus behavior

**Likely files**
- `src/app/state.rs`
- `src/app/reducer.rs`
- `src/ui/render.rs`
- `src/ui/widgets/sidebar.rs`

## Chunk 10 - Pull request awareness

**Goal**
Show pull request state for the selected workspace without making the dashboard brittle.

**SPEC coverage**
- R17
- A16-A17
- pull-request-panel invariant

**Acceptance checklist**
- [ ] Selected workspaces can resolve compact PR state when local tooling is available
- [ ] PR detail can expand from the compact state
- [ ] Opening in browser and copying URL work when supported
- [ ] The detail panel does not repeatedly auto-reopen after the operator closes it
- [ ] Missing or unauthenticated tooling fails soft

**Architecture guardrails**
- PR lookup, browser open, and clipboard copy remain service concerns
- Cache lookup results by workspace identity, not by transient cursor position
- UI state should distinguish "unknown", "missing", and "available" PR states

**Dominant validation**
- Provider contract tests
- Reducer tests for panel-open and auto-open suppression state
- Graceful-degradation smoke tests

**Likely files**
- `src/services/pull_requests.rs`
- `src/app/state.rs`
- `src/app/reducer.rs`
- `src/ui/render.rs`

## Chunk 11 - Notifications and observability

**Goal**
Close the attention loop and make operational decisions inspectable.

**SPEC coverage**
- R18
- A2
- A18
- observability invariants

**Acceptance checklist**
- [ ] Completion notifications emit when suppression rules do not apply
- [ ] Needs-attention notifications emit when suppression rules do not apply
- [ ] Looking at the relevant pane or agent suppresses redundant notifications
- [ ] Per-agent cooldowns reduce noise
- [ ] Runtime mute and profile switching update UI state
- [ ] Structured run logs, latest-run pointer, and retention cleanup all work

**Architecture guardrails**
- Notification policy is pure or near-pure and unit-testable
- Backend selection and fallback stay outside reducer logic
- Observable events are logged with enough context to debug suppression decisions

**Dominant validation**
- Policy unit tests
- Notification backend contract tests
- Logging service tests
- Shell smoke test for log creation and notification happy paths

**Likely files**
- `src/services/notifications.rs`
- `src/services/logging.rs`
- `src/app/state.rs`
- `src/app/reducer.rs`

## Chunk 12 - Acceptance sweep and contract sync

**Goal**
Prove the main operator journeys end to end and leave the repo contracts matching reality.

**SPEC coverage**
- R19
- A19
- definition of done

**Acceptance checklist**
- [ ] Core operator smoke tests exist for startup, inventory, focus, input, and pane actions
- [ ] The fast quality gate stays green
- [ ] Heavy verification is updated to reflect the real app shape
- [ ] `SPEC.md`, `docs/architecture.md`, and user-facing docs match the implemented behavior
- [ ] Any architectural decisions discovered during the rebuild are captured as ADRs when warranted

**Architecture guardrails**
- Do not treat smoke tests as a replacement for unit and contract coverage
- Contract changes must update docs in the same slice or immediately after
- Validation commands documented in the repo must remain true

**Dominant validation**
- `mise run check`
- `mise run verify`
- smoke suite
- `/plan-sync`
- `/spec-sync`

**Likely files**
- `tests/*`
- `.mise/tasks/*`
- `SPEC.md`
- `docs/architecture.md`
- `README.md`
