---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — native-warning-root-fix

## Problem

Foreman warns about visible Claude Code panes missing native signals even when
some of those panes are actually Codex or Pi sessions. The false positives come
from compatibility recognition using stale pane text without enough runtime
identity. Codex Plan Mode question prompts can also remain displayed as
`working` because the native hook signal does not know the terminal is waiting
for user input; that gap should be represented honestly instead of hidden behind
native-looking terminal heuristics.

## Requirements

### MUST

- Prefer high-confidence runtime identity over stale scrollback when identifying
  Claude, Codex, and Pi panes.
- Preserve native signal protection so stale signals from another harness cannot
  override an already-classified different harness.
- Keep native integration status pure to provider hook/file signals. A native
  `working` signal must remain `working` even when terminal text contains
  attention-looking phrases.
- Validate the fix with simulated tmux E2E coverage that reproduces the mixed
  Claude/Codex/Pi warning scenario.

### SHOULD

- Include pane-level evidence in doctor output for missing native signal
  warnings.
- Keep runtime process inspection best-effort and non-fatal.

## Constraints

- Do not change native hook file formats.
- Do not automatically kill panes or delete stale native files.
- Do not add modular signal-source config in this slice; preserve existing
  compatibility mode and keep native hook-only.
