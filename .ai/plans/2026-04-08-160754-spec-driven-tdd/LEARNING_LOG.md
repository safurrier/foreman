---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises.
---

# Learning Log

## 2026-04-08 16:07 - Initial scoping

The current crate is still a scaffold, so the safest path is spec-first and
test-first rather than trying to wire tmux, Ratatui, and harness integrations
all at once.

## 2026-04-08 16:07 - Skill-guided architecture choice

`ratatui-tui` makes this a clear Shape B app: multiple panes, persistent
session, navigation-heavy, command/action/reducer/effects structure, and a
visible focus and mode model.

## 2026-04-08 16:07 - UI skill adaptation

The loaded UI/UX skills are web-centric, so only the transferable rules should
shape the TUI:

- keyboard navigation must feel instant
- one focal point at a time
- progressive disclosure over crowded panels
- status must be legible even without color

## 2026-04-08 16:18 - Repo validation note

The repo-level documentation validators currently fail for pre-existing issues in
`AGENTS.md` and `docs/architecture.md`. The new plan files are outside `docs/`
and do not add validator-detectable failures of their own.

## 2026-04-08 16:31 - Architecture-first adjustment

Before implementation, it was worth sketching the finished operator experience
and adding a SPEC traceability checklist. That makes the chunk order easier to
defend and keeps the rebuild from drifting into "just make it work" coding.

## 2026-04-08 16:36 - Repo contracts now line up with the plan

After fixing the AGENTS reference and rewriting `docs/architecture.md`, the
repo-level reference and frontmatter validators passed again. That matters
because the architecture sketch is now part of the repo's checked documentation,
not just a temporary planning note.

## 2026-04-08 16:58 - Chunk 1 held up under code

The first slice works as intended when kept narrow:

- a public `app` module with command, action, state, reducer, and fixture seams
- selection stored by logical target ID instead of sidebar index
- reducer reconciliation after refresh, collapse, and filter changes

That validated the core plan choice: put the hard invariants in pure state code
first, and postpone tmux and Ratatui integration until the reducer semantics are
stable.

## 2026-04-08 17:10 - Remaining chunks need explicit gates

After the first commit, the next useful planning pass was not more architecture
theory. It was turning the remaining chunks into concrete acceptance and
validation gates so each slice has a clear definition of done before coding
starts.

## 2026-04-08 17:29 - Chunk 2 validation pattern is worth repeating

Chunk 2 used both unit tests and real subprocess-driven integration tests
against the compiled `foreman` binary. That is the right pattern for future
chunks too: keep pure logic in unit tests, but add at least one real e2e-style
path whenever the feature crosses a process or adapter boundary.

## 2026-04-08 17:48 - Chunk 3 stayed pure on purpose

The render shell slice worked best as a pure Ratatui layer with buffer tests,
not as a half-fake interactive app. That keeps the architecture honest:
rendering can evolve and be tested now, while the eventual event loop and tmux
integration still arrive as separate chunks.

## 2026-04-08 18:35 - Chunk 4 needed both fake and real tmux tests

The tmux slice was the first place where unit tests alone stopped being
convincing. The useful pattern was:

- fake-backed adapter tests for grouping, capture fallback, and recognition
- reducer/state assertions for default visibility and toggle behavior
- a dedicated real tmux fixture server for the startup path and binary-level log
  proof

That combination gives fast local feedback without pretending the subprocess and
socket boundary are already covered.

## 2026-04-08 19:18 - Popup behavior belongs in effects, not rendering

Chunk 5 clarified an important boundary: popup auto-exit is not a UI rendering
concern and not a tmux adapter concern by itself. The reducer needs to emit a
focus effect that carries `close_after` intent, and the eventual runtime loop
will honor that after successful tmux focus. That keeps popup behavior testable
before the full interactive event loop exists.

## 2026-04-08 20:02 - Compatibility status needed a separate observation layer

Chunk 6 worked cleanly only after separating tmux transport from compatibility
interpretation. The adapter now gathers pane observations, `src/integrations/`
derives compatibility snapshots from those observations, and reducer refresh
logic applies the debounce policy during inventory replacement. That split keeps
status heuristics pure and makes the eventual native-precedence work a
targeted follow-up instead of another tmux refactor.

## 2026-04-08 20:41 - Native precedence also needed its own overlay seam

The first native integration worked best as a post-discovery overlay, not as a
special case inside `tmux.rs`. Compatibility discovery still builds the base
inventory. `src/integrations/claude.rs` then overlays native Claude signals on
top, replaces the normalized snapshot when present, and leaves compatibility in
place when the native file disappears or errors. That keeps precedence
enforcement in one place and made the tmux + file-shim E2E test straightforward.

## 2026-04-08 22:18 - Direct actions needed target-aware modal state

Chunk 8 stayed clean once the modal state owned both the target identity and
the draft text. `AppState` now keeps input composition separate from
rename/spawn/confirm modals, so reducer tests can assert exact `Effect` payloads
without touching tmux. That also made the real tmux smoke tests straightforward:
the adapter only has to prove send, rename, spawn, and kill work against a live
server, while the reducer remains the source of truth for confirmation and draft
behavior.

## 2026-04-08 23:01 - Flash labels needed to be fixed-width, not prefix-based

Chunk 9 only became reliable once flash labels stopped mixing one-character and
two-character forms in the same invocation. Fixed-width labels avoid the
classic `a` versus `aa` ambiguity, which means the reducer can treat a complete
typed label as an immediate selection or focus action without waiting on extra
confirmation. Search used the same lesson in a different direction: the query
and restore-selection target belong in reducer-owned state so cancel can always
return the cursor to a stable logical target.

## 2026-04-09 09:12 - PR detail state needed workspace identity, not cursor state

Chunk 10 stayed clean once pull request state stopped being thought of as
"whatever the current row shows." The durable identity is the selected
workspace path resolved from tmux pane working directories. The cache keys, the
auto-open suppression set, and the detail panel state all need to follow that
workspace identity rather than a transient sidebar position. The other useful
split was between auto-open and manual-open behavior: closing a discovered PR
should suppress future auto-open for that workspace, but a manual reopen should
still survive later refreshes.

## 2026-04-09 10:06 - Notification policy should follow visible state, not raw observations

Chunk 11 only settled once notifications were tied to the same debounced,
operator-visible status model as the dashboard. The first refresh from working
to idle is still a debounce step, so completion notifications should not fire
until the visible pane status actually flips to idle. The other useful split
was policy versus delivery: suppression rules and cooldowns stay pure and
testable, while backend fallback and shell execution stay in the notification
service.

## 2026-04-09 10:21 - Acceptance work surfaced a real runtime gap

The observability slice itself landed cleanly: header stats now come from a
small service seam, operator-visible soft failures live in `AppState`, and the
binary/bootstrap path logs both. The more important discovery was broader:
the crate has a real reducer, renderer, and smoke suite, but the compiled
binary still bootstraps and exits instead of running the persistent interactive
dashboard loop. That means chunk 12 cannot honestly be called complete until
the event loop and effect executor exist as first-class runtime code.

## 2026-04-09 14:40 - Runtime delivery closed the last product gap

The runtime slice stayed clean once it reused the existing
`Command -> Action -> Reducer -> Effects -> Render` path instead of inventing a
second control flow just for the binary. `src/cli.rs` now prepares typed
bootstrap state, and `src/runtime.rs` owns terminal setup, event polling,
redraw cadence, and effect execution.

The useful E2E lessons were both test-shape issues, not product regressions:

- detached tmux panes can return a successful but blank alternate-screen
  capture, so the fixture needed a primary-screen fallback for dashboard
  assertions
- runtime input smoke tests must target a pane that is actually recognized as
  an agent, otherwise the visibility rules correctly hide it and the navigation
  assertions fail

Those failures were worth keeping because they proved the runtime loop and the
visibility contract were finally meeting at a real process boundary.
