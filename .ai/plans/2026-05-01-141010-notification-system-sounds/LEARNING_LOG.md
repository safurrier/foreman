---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-05-01

- `alerter --sound foreman-spike-reallygood` and
  `alerter --sound foreman-spike-reallygood.aiff` both delivered after copying
  `reallygood.aiff` into `~/Library/Sounds`. The implementation should use the
  basename form so config does not depend on file extensions.
- The DND-respecting path is to keep custom sounds in Notification Center
  delivery. Direct `afplay` remains useful but should be documented as a direct
  playback mode that may bypass Focus.
- The prior random sound path used a salt parameter but did not advance it.
  Advancing the salt on each random resolution keeps both file-directory sounds
  and notification-sound names from being artificially sticky.
- Focus validation succeeded for the Notification Center path: the same
  `alerter --sound foreman-completed-reallygood` command produced audible custom
  sound with Focus off and no audible sound with Focus on, while still
  delivering the notification.
