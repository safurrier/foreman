---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — pi-native-integration

## Approach

Use the existing shared native-signal overlay rather than inventing a Pi-only
path. Pi does not fit the Claude/Codex stdin-hook pattern cleanly here, so the
native transport is a thin Pi extension that calls a new `foreman-pi-hook`
companion binary on Pi lifecycle events. The binary resolves the pane and
writes the same atomic per-pane JSON signal files that Claude and Codex already
use.

## Steps

1. Extend shared types and config/runtime wiring for Pi.
2. Add Pi compatibility recognition and status heuristics with a safer matcher
   than a bare `pi` substring.
3. Add the Pi bridge module and `foreman-pi-hook` binary.
4. Add Pi-native tests:
   bridge CLI tests,
   bootstrap/native precedence tests,
   real Pi-binary E2E through tmux and notifications.
5. Sync the top-level spec, README, architecture record, and this work unit.
6. Run `cargo test`, the live Pi opt-in E2E, `mise run check`, and
   `mise run verify`.
