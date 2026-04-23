---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — tui-ux-diagnostic

## Problem

Foreman's core functionality is present, but the current terminal UX is still
hard to read and hard to operate. The sidebar hierarchy is not self-explanatory,
focus and navigation affordances are weak, feature discovery is poor, and the
screen does not communicate enough about what actions are available or what
state the operator is in.

The next pass should not start from aesthetic taste alone. It should begin with
a real diagnostic of the current app running in tmux, using the actual keyboard
flows and feature surfaces, then convert that evidence into a theme and UX
improvement plan with explicit validation.

## Requirements

### MUST

- Run a live diagnostic against the current `foreman` binary in tmux rather than
  relying only on code inspection.
- Walk through the main operator flows: startup, inventory reading, selection,
  focus movement, direct input, search, flash navigation, popup focus, pull
  request surface, and notification-related state where practical.
- Record concrete UX failures and friction points with enough detail to drive
  implementation slices.
- Produce a prioritized implementation plan for a UX/theme pass, not just a list
  of complaints.
- Include theme support in scope, with a clear architecture for semantic colors
  and non-color fallback.
- Preserve current product contracts in `SPEC.md` and system boundaries in
  `docs/architecture.md`.

## Observed Diagnostic Findings

The live walkthrough produced a few clear categories of failure that the next
implementation pass should treat as first-class requirements:

- Information hierarchy is weak.
  The sidebar repeats generic labels (`Session`, `Window`, `Pane`) but does not
  surface the information operators actually need first: harness kind, status
  summary, or why one row matters more than another.
- The layout is not resilient at smaller terminal sizes.
  At roughly 80x24, help and other overlays clip into adjacent panes instead of
  presenting as legible, self-contained surfaces.
- Discoverability is poor.
  The footer and help panel expose only part of the command surface, and common
  flows like direct input do not read as clear calls to action from the idle
  state.
- State changes are too subtle.
  Pull request detail toggles, filter toggles, and notification-profile changes
  do not create enough visible confirmation.
- Theme support does not exist yet.
  Styling is still hard-coded in `render.rs`, so the app has no semantic palette
  abstraction, no theme selection, and no clean built-in palette or no-color
  fallback story beyond the existing raw text.
- At least one functional bug likely exists in the current UX surface.
  In the live interactive harness, spawn submission worked with `Ctrl+S` but not
  with `Enter`, despite the modal copy promising both. That needs dedicated
  verification during the next pass.
- Foreman can accidentally show its own dashboard as a managed target when the
  inspected tmux socket contains a Foreman pane.
  That is likely recursive compatibility detection and should be treated as a
  product bug, not just a diagnostic oddity.

### SHOULD

- Capture pane output snapshots from the live walkthrough and, if practical,
  generate visual screenshot artifacts.
- Identify where the current product shape should become more dashboard-like vs
  where it should become more input-first.
- Distinguish between usability defects, information hierarchy problems,
  discoverability problems, and visual polish problems.
- Propose validation layers for the UX work, including render tests and live tmux
  smoke coverage for changed flows.
- Sync plan/docs if the diagnostic reveals repo guidance gaps.

## Constraints

- Do not redesign blindly from static code or opinion alone.
- Do not let theme work break monochrome usability or status communication.
- Do not add motion or ornament that slows down repeated keyboard flows.
- Treat terminal constraints as real product constraints: no hover, limited
  color reliability, and limited spatial resolution.
- Use the current ratatui architecture rather than replacing the whole runtime.
