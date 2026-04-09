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
