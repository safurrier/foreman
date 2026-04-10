---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — codex-hook-native

## Approach

Start with Codex's official hook events rather than its terminal text. The
current docs expose `SessionStart`, `UserPromptSubmit`, `PreToolUse`,
`PostToolUse`, and `Stop`. That is enough to map turn start and turn completion
cleanly; the missing design question is how much of `needs attention` can be
native versus compatibility-backed.

Plan to mirror the Claude bridge shape where useful: a companion bridge binary,
typed config, atomic per-pane signal writes, and tmux-backed E2E. But keep the
event mapping Codex-specific.

One validation wrinkle matters: the real `codex exec` binary emitted hook
events reliably in supported non-interactive mode, but in local validation the
same command inside a tmux TTY did not emit hooks. The plan should therefore
keep two proofs:
- real Codex binary E2E at the hook-bridge boundary
- separate Foreman tmux/dashboard E2E for runtime behavior around native files

## Steps

1. Confirm the exact Codex hook events and JSON fields we can rely on.
2. Design Codex-native state mapping and explicit fallback behavior for missing
   `needs attention` and `error` semantics.
3. Add a Codex hook bridge and bootstrap wiring.
4. Add bridge tests, bootstrap precedence tests, and a real Codex-binary E2E
   for hook delivery.
5. Sync docs, SPEC traceability, and validation logs.
