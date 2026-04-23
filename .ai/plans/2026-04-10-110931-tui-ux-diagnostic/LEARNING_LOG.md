---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-10

- The first live harness used panes that ended in `sleep`, which was enough for
  navigation review but not enough to prove direct input or pane actions. For UX
  validation, representative panes need to stay interactive.
- Detached tmux alternate-screen capture was unreliable for this app. Normal
  pane capture (`capture-pane -p -J`) was the stable text artifact source.
- `vhs` is viable for reproducible visual TUI review here, but it is much easier
  if Foreman runs outside the inspected tmux socket. Otherwise the app can end
  up reviewing another Foreman dashboard pane instead of only the agent panes.
- A fake-but-interactive shell banner like `Claude Code ready` is enough to
  validate most operator flows without paying the cost of running a real harness.
- The spawn modal needs a dedicated regression check. In the live interactive
  walkthrough, `Ctrl+S` spawned correctly, while `Enter` did not submit the same
  draft.
- Foreman appears vulnerable to recursive compatibility detection when its own
  pane preview contains harness-like text. That should be fixed before a broader
  UX polish pass lands, otherwise screenshots and real usage both stay noisy.
- The self-detection heuristic must not depend on one exact header string. Once
  the header copy changed, Foreman started classifying its own dashboard again.
  The durable signal is the combination of the Foreman header plus the shared
  dashboard surfaces (`Targets`, `Compose`), not one old preview token.
- Generic tmux window names like `sh` are a real usability tax. Falling back to
  a derived navigation title from the pane workspace makes the tree materially
  easier to scan without changing the underlying tmux model.
- The VHS tape is part of the validation surface, not just marketing. It broke
  once the CLI lost an old `--log-dir` flag and once the initial paint took
  longer than the original screenshot delay. Keeping the tape runnable is part
  of keeping the UX evidence honest.
- Named palette themes are easier to reason about than generic labels like
  `default` or `high-contrast`. Operators already recognize `catppuccin`,
  `gruvbox`, `tokyo-night`, `nord`, and `dracula` from other TUIs, while
  `terminal` and `no-color` should stay separate fallback modes rather than
  pretending to be branded themes.
- The fast gate caught one real regression after the theme refactor: nested
  `format!` in the wide header path tripped Clippy. The repo's strict `check`
  gate is doing useful design-pressure work here, not just style policing.
