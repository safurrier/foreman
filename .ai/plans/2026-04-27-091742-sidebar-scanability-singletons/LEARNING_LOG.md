---
id: plan-learning-log
title: Learning Log
description: Dev diary for this unit of work.
---

# Learning Log

## 2026-04-27

- Screenshot review: normal agent view is still too topology-heavy. Single-window/single-pane sessions show three rows for one operator target.
- Existing `mise run verify-ux` is the right mid/heavy visual gate: it runs focused render regressions, runtime smoke, perf smoke, and refreshes VHS screenshots/GIF under `.ai/validation/ux`.

## Implementation Notes

- `Inventory::visible_targets` and `AppState::build_base_visible_entries` must change together; otherwise keyboard selection can land on rows that are no longer rendered.
- Default `agents-only` scope is the right place to optimize for operator scanability; topology-oriented filter modes preserve raw tmux windows for debugging.

## Validation Retrospective

- Full check was necessary because several integration-style tests encoded exact sidebar depth.
- `verify-ux` is the correct mid/heavy visual gate for screenshot and GIF evidence before PR updates.
- The slice stayed display-model focused; future sidebar polish can build on the same visible-target contract.
