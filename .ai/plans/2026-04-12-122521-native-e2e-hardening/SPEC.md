---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — native-e2e-hardening

## Problem

Real harness E2E readiness is too implicit. The repo can skip native lanes
without failing closeout, and the Codex real E2E depends on hook support that
older PATH-installed Codex builds do not provide. On this machine, `codex`
resolved to `0.92.0` from `~/.local/bin` even though a newer `0.120.0` install
already existed elsewhere on disk. The result is misleading “native E2E not
done” state: the machine looks configured, but the repo does not explain why
the real Codex lane fails, and the task runner does not gate against it.

## Requirements

### MUST

- Add a repo-local preflight that checks tmux availability and the provider CLI
  seams needed for real native E2Es.
- Fail fast when strict native closeout is requested without the required env
  flags enabled.
- Fail fast when Codex on PATH is too old for the hook-based Codex E2E.
- Keep the actual real E2E tests as the source of truth for done; preflight can
  reduce setup churn but cannot replace them.
- Leave a clear operator-facing path for “how do I make native E2E ready on
  this machine?”

### SHOULD

- Stabilize the Claude real E2E if the current machine is correctly configured
  but the test is timing-sensitive.
- Keep opt-in local workflows available for single-provider debugging without
  weakening strict closeout.
- Surface the most actionable remediation text in both docs and task output.

## Constraints

- Do not weaken the requirement that real harness work is only done after the
  actual real E2Es run.
- Use the Codex version and hook surface that the current official docs support.
- Avoid brittle host-specific assumptions beyond the PATH-resolved binary that
  the test runner will actually execute.
