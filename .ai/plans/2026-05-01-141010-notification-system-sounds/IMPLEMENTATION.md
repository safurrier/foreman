---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — notification-system-sounds

## Approach

Add a new sound-source syntax that stays on the system notification path:
`notification-sounds:<prefix>`.

At config load time, Foreman scans `~/Library/Sounds` for AIFF/AIF/CAF/WAV files
whose basenames start with the prefix. At notification time it resolves the
selected basename as a system sound, which lets `alerter --sound` deliver the
audio through Notification Center instead of spawning `afplay`.

Existing direct file and directory sounds remain unchanged.

## Steps

1. Probe `alerter --sound` with copied user sounds.
2. Add prefixed sound-source parsing and resolution.
3. Keep direct file/directory playback behavior intact.
4. Add tests for prefix scanning, alerter delivery, and random salt advancement.
5. Document setup and Focus behavior.
6. Run focused tests, `mise run check`, local install, and manual macOS smoke.
