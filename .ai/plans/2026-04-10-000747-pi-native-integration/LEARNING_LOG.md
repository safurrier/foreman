---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-10

- Pi needed a different native transport than Claude and Codex. The clean seam
  here was not a fake stdin hook parser, but a small Pi extension that calls a
  Foreman companion binary on lifecycle events.
- A bare `pi` substring was too loose for recognition because the shared matcher
  is substring-based. The fix was to recognize Pi by exact command basename when
  possible and use narrower preview tokens for shell-loop fallbacks.
- The strongest live proof was a tmux-driven dashboard test that sends a real
  prompt into a real Pi process, observes extension-driven native signals, and
  waits for a completion notification.
