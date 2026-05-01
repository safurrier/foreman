---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — notification-system-sounds

## Problem

Foreman supports custom audio directories by playing selected files with
`afplay`. That keeps the current custom sound experience, but direct playback
can bypass macOS Focus / Do Not Disturb. A local spike confirmed `alerter
--sound <name>` accepts custom AIFF files copied into `~/Library/Sounds`, which
keeps playback on the macOS notification path.

## Requirements

### MUST

- Add an opt-in sound source that selects custom notification sound names from
  `~/Library/Sounds` by prefix.
- Preserve existing system sound, file, directory, random, sequential, and
  `none` behavior.
- Route the new source through existing `ResolvedNotificationSound::System` so
  alerter/osascript own sound delivery and Foreman does not call `afplay`.
- Document the setup and the DND/Focus tradeoff clearly.

### SHOULD

- Validate with unit tests and a local alerter smoke.
- Keep the config syntax compact enough to edit by hand.

## Constraints

- Do not implement brittle macOS Focus state detection.
- Do not move user sound files automatically.
- Do not remove direct `afplay` support for existing profiles.
