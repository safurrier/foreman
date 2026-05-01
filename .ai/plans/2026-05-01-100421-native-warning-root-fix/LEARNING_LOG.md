---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-05-01

- Runtime identity needs real process metadata, not just tmux
  `pane_current_command`: macOS tmux reports Python/Node wrapper names, while
  the full process command line still contains discriminators like
  `@openai/codex`.
- Process inspection should be done once per inventory load. A per-pane `ps`
  call would solve correctness but regress Foreman's refresh cost.
- Codex native hooks can accurately say `working` while Codex's terminal UI is
  asking a Plan Mode question. Foreman should show the hook state as-is instead
  of labeling heuristic terminal inference as native.
- Adding runtime process inspection made one notification E2E expose a real
  race: the test flipped a native fixture to idle before Foreman had observed
  the initial working state. The fix is to wait for the visible pane state
  before mutating native files in simulated runtime tests.
- Live bootstrap after the fix reported Codex native signals as active and left
  only the real missing Claude native-signal warning, matching the intended
  root-fix behavior.
- Codex and Pi native `working` signals were falsely promoted to attention when
  compatibility scanned full scrollback and saw stale words like `question`,
  `confirm`, or `waiting for input`. The root fix is to remove terminal
  heuristics from native overlays entirely; unsupported attention states should
  remain unsupported until providers emit hook events for them.
