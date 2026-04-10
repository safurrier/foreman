---
id: plan-todo
title: Task List
description: >
  Checkable tasks for this unit of work. Check off as you go.
  See _example/ for a reference.
---

# TODO — full-spec-gap-closure

## Validation ergonomics
- [x] Add `.dockerignore` so Docker build context excludes `target/`, VCS data, plan artifacts, and logs
- [x] Record the Colima/Docker prerequisite and the now-green heavy validation result

## CLI logging contract
- [x] Add a public debug logging flag to `src/cli.rs`
- [x] Thread debug/log-level state through `RuntimeConfig`
- [x] Make `RunLogger` honor the configured verbosity without changing existing info logs
- [x] Add tests for CLI parsing and debug-log behavior

## Notification config contract
- [x] Extend `AppConfig` with notification cooldown configuration
- [x] Extend `AppConfig` with notification backend preference/order
- [x] Extend `AppConfig` with active notification profile selection
- [x] Keep sensible defaults so existing configs still parse

## Runtime wiring
- [x] Initialize notification state from config defaults
- [x] Build the notification dispatcher from configured backend order
- [x] Add tests proving config changes runtime notification behavior

## Integration preference contract
- [x] Extend config with per-harness integration preference
- [x] Make Claude honor `auto`, `native`, and `compatibility` preference
- [x] Add tests for forced compatibility and native-preferred behavior

## Final sync
- [x] Update `SPEC.md`, `README.md`, and `docs/architecture.md`
- [x] Run `cargo test`
- [x] Run `mise run check`
- [x] Run `mise run verify`
