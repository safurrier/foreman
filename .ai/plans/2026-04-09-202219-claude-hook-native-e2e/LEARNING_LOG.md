---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-09

- The official Claude Code hook surface was sufficient. `UserPromptSubmit` and
  `Stop` both fire under `claude -p`, so Foreman did not need transcript
  parsing or terminal scraping to prove a real native path.
- The real integration gap was operational, not architectural. Foreman already
  knew how to consume native signal files; it just lacked a supported way to
  produce them from actual Claude sessions.
- The strongest E2E shape was a shell loop inside a tmux pane that runs real
  `claude -p` calls from dashboard-sent input. That kept the test close to the
  operator workflow while staying deterministic enough for a manual gate.

## Retrospective

- Matched plan: companion binary, config/runtime default path, binary bridge
  tests, real opt-in E2E, and doc sync.
- Diverged slightly: the first real-Claude assertion failed on a log-message
  name, not behavior. The fix was to assert the actual logged event
  `notification_backend_selected`.
- One-shot next time would improve if the repo had an explicit place for manual
  E2E evidence so the ignored test and its validation command were easier to
  discover.
