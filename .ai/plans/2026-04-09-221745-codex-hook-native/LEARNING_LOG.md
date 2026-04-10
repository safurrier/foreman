---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-09

- Codex is a real candidate for native integration. The official docs now
  expose a hook system with `UserPromptSubmit`, `PreToolUse`, `PostToolUse`,
  `SessionStart`, and `Stop`, which is stronger than the earlier assumption
  that we might need to rely on `codex exec --json` alone.
- The likely gap versus Claude is `needs attention`. Codex's hook docs are
  strong on turn and tool lifecycle but do not obviously expose a Claude-style
  notification event for "waiting on the operator", so we may need a hybrid
  native-plus-compatibility model there.
