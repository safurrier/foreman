---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — notification-sound-click

## Problem

Foreman notifications currently work on macOS but the visible message is noisy:
it includes the full workspace path, pane id, and integration-source details in
the body. tmuxcc has a more useful shape: concise notification copy,
configurable sounds, and macOS click-through back to the relevant tmux pane.

## Requirements

### MUST

- Keep notification title/body concise and readable on macOS.
- Preserve enough context to identify the target agent and tmux location.
- Add a notification backend that can use macOS `alerter` click actions to
  select the relevant tmux window/pane.
- Add configurable notification sounds for completion and attention events.
- Support sound files from directories so existing tmuxcc ITYSL sound folders
  can be reused.
- Keep existing `notify-send` and `osascript` behavior working.
- Add unit and integration-style validation for message formatting, backend
  fallback, sound resolution, and click-through tmux command execution.

### SHOULD

- Prefer `alerter` on macOS when available, then fall back to existing
  backends.
- Use ASCII separators in notification copy to avoid odd rendering.
- Avoid blocking the TUI event loop while waiting for a notification click.
- Document the new config keys.

## Constraints

- Do not regress existing config files; new fields must have defaults.
- Do not require sound files to exist for the default configuration.
- Do not put full workspace paths in visible notification body text.
- Runtime tests should not replace the `tmux` binary used by Foreman's tmux
  adapter.
