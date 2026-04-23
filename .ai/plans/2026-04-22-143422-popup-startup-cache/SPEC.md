---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
---

# Specification — popup-startup-cache

## Problem

Popup startup is better after the loading-first slice, but operators still pay for a live tmux refresh before they see meaningful session state. Reopening Foreman in popup mode is a repeat action, so a tiny persisted inventory snapshot can improve first paint further if it stays clearly stale and cheap to maintain.

## Requirements

### MUST

- Load a small persisted inventory snapshot before the first live tmux refresh when the cache is fresh.
- Mark cached startup state clearly in the UI so it is not mistaken for live truth.
- Replace cached state with live tmux inventory as soon as refresh completes.
- Keep destructive behavior, notifications, and focus actions grounded in live refresh rather than cache-only claims.
- Add profiling coverage that proves cache writes are not frequent enough or slow enough to become a new hitch.
- Pass the full validation ladder, including strict real native verification.

### SHOULD

- Key cache files by tmux server identity so alternate sockets do not bleed into each other.
- Reuse the existing state and inventory model instead of inventing a second parallel cache-specific model.
- Skip unnecessary cache writes when the inventory has not materially changed.

## Constraints

- Keep the cache small and file-based. Do not add a daemon or persistent background service.
- Keep cached state as startup sugar only; live tmux refresh remains the source of truth.
- Avoid adding enough synchronous write pressure to undo the recent refresh-performance work.
