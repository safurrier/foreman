---
id: plan-traceability
title: Remaining 1.0 Traceability
description: >
  Checklist mapping the remaining full-v1 gaps back to the top-level spec.
---

# Remaining 1.0 Traceability — full-spec-gap-closure

- [x] **CLI debug logging mode**
  Root spec: `SPEC.md` CLI contract
  Status: `foreman --debug` now enables debug-level run logging and has binary
  coverage in `tests/cli_config.rs`

- [x] **Notification cooldown and backend preference in config**
  Root spec: `SPEC.md` config contract
  Status: cooldown and backend order now live in typed config and are covered
  by config, service, and runtime E2E tests

- [x] **Active notification profile from config**
  Root spec: `SPEC.md` config contract
  Status: startup profile now seeds `NotificationState` and has config plus
  runtime suppression coverage

- [x] **Per-harness integration preference**
  Root spec: `SPEC.md` config contract
  Status: Claude now honors `auto`, `native`, and `compatibility` preference
  with integration coverage in `tests/claude_native.rs`

- [x] **Heavy validation loop**
  Root spec: `SPEC.md` task contract and definition of done
  Status: `mise run verify` now passes locally after restoring the Colima
  runtime behind the active Docker context
