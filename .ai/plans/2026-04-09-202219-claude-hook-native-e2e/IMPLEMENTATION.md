---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — claude-hook-native-e2e

## Approach

Add a dedicated companion binary, `foreman-claude-hook`, instead of overloading
the main dashboard CLI. The bridge reads official Claude Code hook JSON on
stdin, resolves the pane from `TMUX_PANE` or `--pane-id`, maps supported events
to Foreman statuses, and writes atomic per-pane signal files.

Also tighten runtime/config so Claude native mode has a real default signal
directory and an additive config override. Prove the bridge at three layers:
unit mapping tests, binary bridge tests, and an ignored real-Claude tmux E2E.

## Steps

1. Add typed config support and a default native signal directory.
2. Add the Claude hook bridge module and companion binary.
3. Add binary tests for hook mapping and path resolution.
4. Add an ignored real-Claude dashboard E2E driven by dashboard input.
5. Sync README, architecture docs, and validation logs.
