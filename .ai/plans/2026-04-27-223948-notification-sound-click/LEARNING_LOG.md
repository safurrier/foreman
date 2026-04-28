---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-27

- Compared `../tmuxcc/src/notifications.rs` before implementation. Useful
  patterns: concise body, per-event sound config, directory-backed sound
  rotation, `alerter` click actions, and background click handling that runs
  `tmux select-window` then `tmux select-pane`.
- Kept runtime integration coverage from replacing `tmux` on PATH. Click-through
  targeting is covered with injected fake programs in unit coverage; runtime
  notification coverage still uses the real tmux fixture.
- Local Foreman config now reuses the existing tmuxcc ITYSL sound directories
  with `sound_profile = "itysl"` and `alerter` first in backend order.
- Manual diagnosis found the first click-through implementation only ran
  `select-window` and `select-pane`. That can select the target inside tmux but
  does not reliably move the attached client across sessions. The click path now
  mirrors Foreman's normal focus behavior: resolve the live pane target from
  tmux, run `switch-client` to the target session when a client exists, then
  select the window and pane.
- Real click behavior still needed more visibility because the alerter click
  handler runs in a detached thread. Added `notification_alerter_*` and
  `notification_tmux_*` log lines to the active `latest.log` so the next manual
  click records alerter output, sound spawning, resolved tmux targets, and tmux
  command statuses.
- Tested a modal macOS dialog as an accessibility alternative. It likely works
  better for clickable-element overlays, but the UX is too intrusive for normal
  Foreman notifications. Kept `alerter` and renamed the action to `Open tmux
  pane` for clearer VoiceOver/action-menu wording.
