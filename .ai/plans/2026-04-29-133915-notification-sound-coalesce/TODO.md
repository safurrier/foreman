---
id: plan-todo
title: Task List
description: >
  Checkable tasks for this unit of work. Check off as you go.
  See _example/ for a reference.
---

# TODO — notification-sound-coalesce

## Planning
- [x] Create feature branch
- [x] Create `.ai/plans` slice with spec, implementation, TODO, validation, and learning log
- [x] Scope behavior: same-refresh coalescing plus audible gating

## Implementation
- [x] Add audible flag to notification requests
- [x] Make dispatcher suppress sound for inaudible requests
- [x] Add grouped notification request builder
- [x] Batch same-kind notifications per inventory refresh
- [x] Preserve cooldown entries for every contributing pane

## Tests
- [x] Unit test grouped completion request copy and target
- [x] Unit test inaudible command dispatch has empty sound
- [x] Reducer test for two completions becoming one notify effect
- [x] Focused notification test run

## Validation
- [x] Run `mise run check`
- [x] Update `VALIDATION.md`
- [x] Update `LEARNING_LOG.md` retrospective

## PR
- [x] Commit branch changes
- [x] Push branch
- [x] Open PR
- [x] Poll CI and reviews
