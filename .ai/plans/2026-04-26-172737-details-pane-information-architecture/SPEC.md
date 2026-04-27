---
id: plan-spec
title: Task Specification
description: Requirements and constraints for this unit of work.
---

# Specification — details-pane-information-architecture

## Problem

The Details pane mixes alerts, startup cache state, event summaries, selected target metadata, PR state, diagnostics, and pane output in one linear stream. Operators have trouble distinguishing what each thing is, especially in popup mode and compact layouts.

## Requirements

### MUST

- Update root `SPEC.md` so the details pane has an explicit information-hierarchy contract.
- Reframe Details output into stable, named sections that separate overview, selected target, PR, diagnostics, and output.
- Avoid duplicating selected target facts across `Selected` and `Pane details` blocks.
- Rename ambiguous "Recent" event wording so it cannot be confused with pane output.
- Preserve compact-layout behavior and monochrome/no-color usability.
- Add or update render tests for the new section labels and wording.

### SHOULD

- Prefer clearer prose over cryptic abbreviations where space allows.
- Keep render pure and avoid moving I/O into UI code.
- Keep the slice focused on details-pane readability, not a broader layout rewrite.

## Constraints

- Follow existing Ratatui pure-render architecture.
- Do not alter command semantics except text/hints needed by the details pane.
- Validate with focused render tests first, then full repo checks and release/UX validation.
