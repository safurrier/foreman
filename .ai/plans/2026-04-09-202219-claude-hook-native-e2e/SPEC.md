---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — claude-hook-native-e2e

## Problem

Foreman's Claude native integration consumed per-pane JSON files, but the repo
did not ship a first-class bridge from official Claude Code hook events into
those files. That left the native path usable in tests and local experiments,
but not properly productized or proven against a real Claude run.

## Requirements

### MUST

- Ship a supported bridge from Claude Code hook stdin to Foreman's per-pane
  native signal files.
- Resolve the signal directory without requiring hidden test-only flags.
- Keep Claude native mode authoritative over compatibility mode when signal
  files exist.
- Add test coverage for hook-event mapping, bridge-path resolution, and real
  tmux-backed dashboard behavior.
- Keep `cargo test`, `mise run check`, and `mise run verify` green.

### SHOULD

- Provide a default signal directory so the bridge works with minimal setup.
- Document the hook configuration needed for Claude Code.
- Add an opt-in real-Claude E2E that proves dashboard input can trigger a real
  Claude turn and a completion notification.

## Constraints

- Do not move hook parsing or file writing into the reducer or renderer.
- Keep the real-Claude proof opt-in because it depends on local auth and
  network.
- Preserve the existing `foreman` CLI contract while adding the bridge.
