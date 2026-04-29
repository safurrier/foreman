---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — notification-sound-coalesce

## Problem

Foreman currently emits one desktop notification per agent transition during
an inventory refresh. When multiple agents return to idle close together, the
operator can see stacked notifications and hear overlapping completion sounds.
The most disruptive case is sound overlap, but the visible notification stack
is also noisy.

The notification system should stay transition-driven and preserve existing
suppression rules while making bursts easier to absorb.

## Requirements

### MUST

- Prevent overlapping completion or attention sounds when multiple
  notifications are dispatched from the same refresh.
- Coalesce multiple unsuppressed notifications of the same kind from one
  inventory refresh into a single desktop notification.
- Preserve existing selected-pane, muted, profile, and per-pane cooldown
  suppression behavior.
- Preserve backend fallback and logging for notification dispatch.
- Keep notification text concise and avoid full workspace paths in visible
  notification bodies.
- Add tests for same-refresh coalescing and sound suppression.

### SHOULD

- Keep the implementation inside the existing notification policy, reducer,
  and dispatcher boundaries.
- Avoid adding new config until behavior proves it needs user tuning.
- Make grouped notifications actionable by keeping a best-effort target pane
  for click-to-focus backends.
- Keep single-notification behavior unchanged except for the new audible
  marker.

## Constraints

- Do not change harness status detection or native hook behavior.
- Do not add a new notification backend or external dependency.
- Do not block the render loop waiting for sounds to finish.
- Do not make historical `.ai/plans/*` artifacts canonical documentation.
- Validation gate for this slice is focused unit tests plus `mise run check`.
