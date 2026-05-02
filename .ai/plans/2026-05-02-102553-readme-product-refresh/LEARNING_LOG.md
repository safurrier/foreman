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
- User feedback corrected that the README demo should use a real Foreman theme,
  not a VHS-driven terminal palette. The next iteration uses Foreman's
  `tokyo-night` theme and records the tmux popup path documented in the user's
  dotfiles.
- The popup demo initially assumed tmux window `0`, but this environment's tmux
  server starts windows at `1`. Targeting named windows in the isolated demo
  script made the harness independent of tmux base-index settings.
- The final README demo should be ad hoc but realistic: an isolated tmux socket
  running real Codex, Pi, and Claude panes, with synthetic native signal files
  only for deterministic Foreman statuses. Fake echo panes looked staged.
- The "theme is black and white" issue was not Foreman config loading. The
  recording environment had `NO_COLOR=1`, and Ratatui/crossterm respected it.
  `tmux capture-pane -e` proved Foreman emitted no SGR styles until the demo
  explicitly unset `NO_COLOR` and set truecolor env vars.
- Demo fixtures must avoid private-context leaks even when the tmux socket is
  isolated. Do not copy real session/window names into the fixture, run real
  agents from `/tmp` workdirs, isolate Pi's agent/session dirs, and override the
  tmux status bar so hostnames do not appear in the GIF.
