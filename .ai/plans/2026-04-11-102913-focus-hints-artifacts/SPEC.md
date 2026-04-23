---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — focus-hints-artifacts

## Problem

Foreman is now functionally solid, but the operator surface still leaves a few
important gaps:

- the footer is clearer than earlier cuts, but it still reads too much like a
  mode cheat sheet and not enough like “what can I do from this exact focus”
- help explains the current glyphs, but it does not yet explain whether status
  came from native integrations or compatibility heuristics
- CI proves the heavy validation path, but the UX and release artifacts are not
  uploaded as first-class review evidence
- the spec still mentions optional mouse navigation even though the product
  direction is firmly keyboard-first for 1.0

## Requirements

### MUST

- Normal-mode footer hints adapt to focus and current selection instead of only
  mode. The operator should see the next sensible actions for `Sidebar`,
  `Preview`, or `Compose`.
- The help surface explains native vs compatibility status sourcing in plain
  language and clarifies the operator’s current focus.
- Selected pane details show integration provenance in a way that distinguishes
  higher-confidence native status from lower-confidence compatibility
  heuristics.
- Small and wide layouts stay readable; the new hints must not turn the footer
  back into unreadable glyph soup.
- Render tests and live tmux-backed walkthroughs prove the new hints and source
  surfacing.
- CI uploads the UX artifact bundle and release-gauntlet report so PR review can
  inspect the same proof locally and remotely.
- `SPEC.md`, `README.md`, `docs/architecture.md`, `docs/workflows.md`, and this
  plan trail stay synced.
- The top-level spec no longer advertises mouse-assisted navigation or clickable
  footer actions for 1.0.

### SHOULD

- Pane rows or details should stay compact. Put the fuller explanation in the
  preview/help surfaces instead of expanding every sidebar row.
- The focus-aware footer should keep steady-state noise low by separating
  primary actions from secondary view toggles.
- CI artifact names should be stable and obvious so reviewers can find the
  release report and screenshots without guessing.

## Constraints

- Keep the reducer/render split intact. This slice is presentational and should
  not move I/O into the reducer or render path.
- Preserve the two-line footer budget in normal mode.
- Prefer existing artifact directories and validation tasks over inventing a
  second parallel evidence pipeline.
- Preserve monochrome and ASCII-safe behavior.
