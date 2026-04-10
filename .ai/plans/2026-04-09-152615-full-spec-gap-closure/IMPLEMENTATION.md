---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — full-spec-gap-closure

## Approach

Treat this as a contract-completion pass, not a new product rewrite.

The existing runtime, reducer, tmux adapter, and smoke coverage already cover
the core operator flows. The remaining 1.0 work should therefore land in small
vertical slices that extend public configuration and logging behavior while
reusing the current architecture.

The most pragmatic order is:

1. fix heavy-validation ergonomics so `verify` stays cheap enough to rerun
2. close the CLI debug-mode gap
3. close notification-config gaps
4. close per-harness integration-preference gaps
5. finish with doc/spec sync and a full validation pass

## Steps

1. Add a validation-ergonomics slice.
   Goal: keep the heavy loop healthy after the Colima fix.
   Scope: add `.dockerignore`, document the local Docker/Colima prerequisite,
   and record that `mise run verify` now passes locally.

2. Add a public debug-logging slice.
   Goal: satisfy the CLI contract with the smallest useful surface.
   Scope: add `--debug` or equivalent public log-level flag, thread it into
   `RuntimeConfig` and `RunLogger`, and cover it with CLI/unit tests.

3. Add a notification-config slice.
   Goal: stop hardcoding notification defaults in runtime/service code.
   Scope: extend `AppConfig` with cooldown ticks, backend order/preference, and
   active-profile selection using built-in named profiles.

4. Add a runtime-notification wiring slice.
   Goal: make config shape materially affect runtime behavior.
   Scope: initialize `NotificationState` from config defaults, build the
   dispatcher from configured backend order, and add reducer/service/smoke tests
   that prove config changes behavior.

5. Add an integration-preference slice.
   Goal: honor per-harness mode preference where multiple modes exist.
   Scope: add config for harness mode preference and wire Claude to obey
   `auto`, `native`, or `compatibility` selection without disturbing the other
   compatibility-only harnesses.

6. Finish with a spec-sync slice.
   Goal: leave the repo internally consistent.
   Scope: update top-level docs, append validation evidence, rerun
   `cargo test`, `mise run check`, and `mise run verify`.

## Dominant Test Layers

- Validation ergonomics: `docker build`, `mise run verify`
- Debug logging mode: CLI/unit tests plus run-log assertions
- Notification config: config parsing tests, service unit tests, focused smoke
  tests
- Integration preference: integration unit tests plus Claude native/compat
  smoke tests
- Final sweep: full repo validation plus doc validators
