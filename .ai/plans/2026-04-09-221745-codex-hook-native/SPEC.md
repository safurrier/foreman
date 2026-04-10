---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — codex-hook-native

## Problem

Foreman already recognizes Codex panes in compatibility mode, and the top-level
rollout order says Codex CLI is the next native integration after Claude. We
need a first-class Codex native bridge built on Codex's official hook surface,
not another round of terminal parsing.

## Requirements

### MUST

- Add a supported Codex hook bridge path that can drive native `working`,
  `idle`, and `error` states.
- Keep Codex native mode authoritative over compatibility mode when native
  signals are present.
- Preserve compatibility fallback when native hook delivery is unavailable.
- Add tests at bridge, bootstrap, and tmux-backed runtime layers.
- Keep `cargo test`, `mise run check`, and `mise run verify` green.

### SHOULD

- Reuse the Claude bridge architecture where it fits, without forcing a fake
  event model onto Codex.
- Use Codex hook events to improve activity confidence during turns.
- Document the minimal Codex hook config needed for operators.

## Constraints

- Stay within the official Codex hook/event surface.
- Keep reducer and renderer ignorant of hook transport details.
- Be explicit about which operator-facing states are truly native versus still
  backed by compatibility fallback.
