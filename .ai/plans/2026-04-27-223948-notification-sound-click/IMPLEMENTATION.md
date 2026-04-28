---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — notification-sound-click

## Approach

Mirror tmuxcc's useful notification shape while keeping Foreman's existing
configuration model. Notification requests will carry separate title, subtitle,
body, and tmux targeting metadata. Backends will receive a resolved sound for
the event and can decide how to use it. The macOS `alerter` backend will run in
a background thread so the runtime does not wait for click/timeout handling.

## Steps

1. Extend notification config with `alerter`, sound profile fields, and runtime
   propagation.
2. Refactor notification requests to use concise visible copy plus tmux target
   metadata.
3. Add sound resolution for system sound names, file paths, and audio
   directories.
4. Add concrete `alerter`, `osascript`, and `notify-send` backends with
   existing command-backend tests preserved.
5. Update unit and runtime tests.
6. Update `SPEC.md`, docs/config examples, and the plan validation log.
7. Run focused tests, `mise run check`, and any needed release-confidence
   validation.
8. Configure the local user config for the existing tmuxcc ITYSL sound folders
   after repo validation passes.
