---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — notification-sound-coalesce

## Approach

Keep transition detection unchanged, then batch the resulting requests before
the reducer emits runtime effects.

The reducer is the right place to coalesce because it sees the full list of
notification decisions for one inventory refresh. The dispatcher remains
responsible for backend fallback and sound resolution, but each request will
carry whether it is allowed to play sound. This lets grouped notifications play
one sound while any remaining edge-case dispatches can be made silent without
changing backend selection.

No config fields are planned for this first slice. Defaults should become
quiet immediately: same-refresh bursts become one visible notification per
kind, and only audible requests resolve configured sounds.

## Steps

1. Extend `NotificationRequest` with an `audible` boolean.
   - Default all existing constructed requests to `true`.
   - Make the dispatcher resolve `ResolvedNotificationSound::None` when
     `audible` is false.
2. Add request coalescing in `src/app/reducer.rs`.
   - Group unsuppressed requests by `NotificationKind` per refresh.
   - Leave one request untouched when there is only one request for a kind.
   - Combine multiple requests for the same kind into one request with a
     grouped title/body.
   - Still record cooldowns for every pane that contributed to a grouped
     notification.
3. Add grouped notification copy in `src/services/notifications.rs`.
   - Completion: `Foreman: N agents ready`.
   - Attention: `Foreman: N need attention`.
   - Body: one concise line per pane, capped if needed.
4. Add tests.
   - Reducer: two working panes that become idle in one refresh produce one
     notify effect and cooldown entries for both panes.
   - Notification service: grouped request has concise body and keeps a target
     pane.
   - Dispatcher: inaudible requests pass an empty sound to command backends.
5. Run validation.
   - Focused tests for notifications/reducer path.
   - `mise run check`.
6. Update plan artifacts.
   - Mark TODO items complete as they land.
   - Append validation commands and final retrospective.
