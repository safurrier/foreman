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
