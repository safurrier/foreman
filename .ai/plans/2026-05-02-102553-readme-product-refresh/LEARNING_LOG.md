---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-05-02

- The README had accurate material, but it mixed too many reader jobs in one
  file. Splitting an operator guide gives the README room to sell the tool
  without deleting reference details.
- The first VHS render captured local config paths. Setting `FOREMAN_CONFIG_HOME`,
  `XDG_STATE_HOME`, and `FOREMAN_LOG_DIR` in the tape made the GIF safe to
  commit and easier to reproduce.
- The first demo showed setup/config output, which was the wrong product signal
  for the README. A better demo creates an isolated tmux world with fake
  Claude, Codex, and Pi panes plus native signal files, then records the real
  dashboard search and flash-jump surfaces.
- The notification docs needed an operator recipe, not just the TOML reference.
  The useful split is direct file/directory playback for simple local sounds
  versus `notification-sounds:<prefix>` for the macOS notification path.
- The built-in RGB app themes looked too muted in VHS frames. For README media,
  Foreman's `terminal` theme plus a brighter VHS palette made the real dashboard
  read more clearly.
