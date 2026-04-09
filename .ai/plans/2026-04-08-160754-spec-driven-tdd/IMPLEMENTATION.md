---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step TDD approach for building Foreman from the existing spec.
---

# Implementation - spec-driven-tdd

## Approach

Treat Foreman as a Shape B Ratatui app and make the pure state core the center
of the design.

Use these skills in this order:

1. `ratatui-tui` for app shape, state model, command/action/reducer/effects,
   and render purity
2. `testing-core` for test-layer selection and behavior-first assertions
3. `emil-design-eng`, `userinterface-wiki`, and `web-design-foundations` for
   transferable terminal UX rules: instant keyboard feedback, one focal point,
   progressive disclosure, clear grouping, and visible status contrast

The key rule for each slice is:

- one user-facing behavior
- one or more spec invariants
- one dominant test layer

See `EXPLORATION.md` for the finished-app sketch and milestone order.
See `TRACEABILITY.md` for the requirement-to-chunk validation checklist.
See `CHUNK_SPECS.md` for the detailed acceptance and validation gate for each
remaining chunk.

## Target app sketch

The completed app should feel like a stable operator console rather than a demo:

- a dense but readable header
- a fast session/window/pane tree in the sidebar
- a preview pane that explains the selected work
- a multiline input box for direct operator intervention
- overlays and panels that appear only when needed, then get out of the way

The operator should be able to answer three questions quickly:

1. What needs attention?
2. What is still working normally?
3. What can I do right now without leaving the dashboard?

## Delivery order

Build the product in this order:

1. state core and render shell
2. tmux inventory and stable navigation
3. status engine and Claude native integration
4. direct actions and discovery tools
5. pull requests, notifications, and observability
6. acceptance sweep and doc sync

## Test ladder

### 1. Unit tests

Use for:

- reducer transitions
- command-to-action mapping
- selection stability across refreshes and sort changes
- status debounce logic
- native-over-compatibility precedence
- config parsing and migration

### 2. Ratatui buffer tests

Use Ratatui's test backend to verify:

- header/sidebar/preview/input/footer/help visibility
- focus borders and mode visibility
- empty-shell error state
- search and flash overlays
- monochrome-safe status markers and labels

### 3. Adapter contract tests

Use fakes and spies for:

- tmux discovery, capture, focus, send-input, rename, spawn, and kill
- harness integrations
- pull request providers
- notification backends

### 4. Real-environment smoke / E2E tests

Reserve for:

- config-path and init-config flows
- startup logging
- multi-session tmux discovery
- filter defaults and reveal toggles against a live tmux fixture
- popup focus behavior
- direct input and pane operations
- end-to-end notification and pull request happy paths

## Proposed module seams

Start with a structure close to this:

```text
src/
  app/
    command.rs
    action.rs
    state.rs
    reducer.rs
  ui/
    render.rs
    layout.rs
    widgets/
  adapters/
    tmux.rs
  integrations/
    claude.rs
    codex.rs
    gemini.rs
    opencode.rs
  services/
    notifications.rs
    pull_requests.rs
    logging.rs
  cli.rs
  main.rs
```

This keeps business rules away from terminal I/O and makes fake-backed tests
straightforward.

## TDD slices

### Chunk 1 - State spine and invariants

Goal: define `AppState`, `Command`, `Action`, `Mode`, `Focus`, and reducer
semantics before any real tmux work.

Spec coverage:

- Primary state invariants
- Mode precedence
- Selection invariants

Dominant tests:

- reducer unit tests

Exit criteria:

- one-focus invariant is encoded
- selection never points at an invalid target after refresh
- search/help/input/modal precedence is deterministic

### Chunk 2 - CLI and config utility flow

Goal: close the lowest-risk CLI contract first.

Spec coverage:

- A1
- R2 config-path and init-config behavior

Dominant tests:

- CLI unit tests
- shell smoke for `--config-path` and `--init-config`

Exit criteria:

- config path prints and exits
- default config initializes
- runtime overrides parse into typed config

### Chunk 3 - Render shell and empty-state dashboard

Goal: render the main surfaces from pure state, even before real inventory data
exists.

Spec coverage:

- R9 main UI surfaces
- tmux missing soft-fail contract

Dominant tests:

- Ratatui buffer tests

Exit criteria:

- header, sidebar, preview, input, footer, and help render
- focused panel is visually obvious
- mode is visible in the footer/status line
- empty shell shows a visible tmux error state instead of crashing

### Chunk 4 - Tmux inventory and visibility rules

Goal: discover sessions/windows/panes and apply default hiding rules.

Spec coverage:

- A3
- A6
- R4
- R7

Dominant tests:

- adapter contract tests
- reducer unit tests for filtering and collapse state

Exit criteria:

- multi-session inventory loads
- non-agent sessions and panes are hidden by default
- runtime toggles reveal hidden items
- collapse state survives refresh

### Chunk 5 - Keyboard navigation and focus actions

Goal: make the shell usable as a keyboard-first control surface.

Spec coverage:

- A8
- A9
- R10
- R11

Dominant tests:

- command mapping unit tests
- buffer tests for focus movement
- adapter contract tests for tmux focus

Exit criteria:

- keyboard navigation moves through visible targets
- escape behaves by current mode
- popup mode closes after successful focus

### Chunk 6 - Compatibility detection and coarse status

Goal: recognize supported harness panes and compute the coarse operator-facing
status model.

Spec coverage:

- A4
- A7
- R6
- R8

Dominant tests:

- status-engine unit tests
- fake capture contract tests

Exit criteria:

- supported harnesses are recognized
- statuses collapse to working / needs attention / idle / error / unknown
- debounce prevents short-lived flicker

### Chunk 7 - Claude native integration and precedence

Goal: make Claude Code the first native integration and prefer native signals
over compatibility heuristics.

Spec coverage:

- A5
- A11
- native-precedence invariant

Dominant tests:

- unit tests for precedence
- adapter contract tests for fallback behavior

Exit criteria:

- native Claude signals drive status when available
- compatibility mode remains as fallback
- notifications can rely on Claude completion/attention events

### Chunk 8 - Direct input and operational actions

Goal: let the operator act on the selected pane.

Spec coverage:

- A10
- A12
- R12
- R13

Dominant tests:

- reducer tests for modal/confirmation state
- adapter contract tests for send, kill, rename, and spawn

Exit criteria:

- multiline input sends correctly
- kill requires confirmation
- rename and spawn actions succeed through the tmux seam

### Chunk 9 - Search, flash navigation, and sort

Goal: add fast targeting and navigation aids after base navigation is stable.

Spec coverage:

- A13
- A14
- A15
- R14
- R15
- R16

Dominant tests:

- reducer unit tests
- buffer tests for overlays and labels

Exit criteria:

- search filters and restores prior selection on cancel
- flash labels jump correctly, including overflow labels
- sorting preserves logical selection where possible

### Chunk 10 - Pull request awareness

Goal: add a secondary detail surface without destabilizing the dashboard.

Spec coverage:

- A16
- A17
- R17
- pull-request-panel invariant

Dominant tests:

- provider contract tests
- reducer tests for panel-open state

Exit criteria:

- compact PR state appears when available
- detail view expands without repeated auto-reopen
- missing tooling fails soft

### Chunk 11 - Notifications and observability

Goal: close the operator attention loop and make failures visible.

Spec coverage:

- A2
- A18
- Observability invariants
- R18

Dominant tests:

- notification policy unit tests
- backend contract tests
- shell smoke for log creation

Exit criteria:

- completion and attention notifications fire with suppression and cooldowns
- mute/profile switching updates state
- structured logs and latest-log pointer are written

### Chunk 12 - Acceptance sweep and docs

Goal: prove the core operator flows and align docs with reality.

Spec coverage:

- A19
- Definition of done

Dominant tests:

- shell smoke suite
- full local quality gate

Exit criteria:

- `cargo fmt`, `cargo clippy`, and `cargo test` pass
- `mise run check` passes once the task is wired in
- README and architecture docs reflect the implemented behavior

## UX rules to enforce from the start

- Keyboard navigation is instant. No delayed transitions or staged focus moves.
- The focused panel must be visually obvious with borders, title treatment, or
  both.
- The footer/status line always shows mode and key help entrypoint.
- Status cannot rely on color alone; text/icon labels must carry meaning.
- Optional panels stay secondary. The sidebar, preview, and input loop remain
  primary.
