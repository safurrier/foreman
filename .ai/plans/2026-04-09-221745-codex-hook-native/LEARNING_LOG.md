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
- The hook bridge architecture wanted a shared file-signal layer after the
  second native integration landed. Pulling the parser, atomic writer, and
  fallback summary into one `native` helper kept Claude and Codex aligned
  without making the reducer or runtime more abstract than necessary.
- The real Codex binary surfaced an important validation nuance: `codex exec`
  emitted `submit` and `stop` hooks reliably in supported non-interactive mode,
  but the same command inside a tmux TTY did not emit hooks in local testing.
  The durable test shape is therefore split between:
  - real Codex-binary hook E2E
  - separate Foreman tmux/dashboard E2E around native-file consumption
