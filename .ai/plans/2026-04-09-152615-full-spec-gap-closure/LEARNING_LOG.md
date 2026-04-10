---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-09 15:20 - Heavy validation failure was mostly local runtime state

The active Docker context was already correct. The real blocker was simpler:
the `default` Colima profile was stopped, so Docker pointed at a socket path
that did not exist. Starting Colima restored the socket and let `mise run verify`
complete.

The secondary lesson is build ergonomics. The Docker build works now, but it
ships a multi-GB context into the daemon because the repo lacks a focused
`.dockerignore`. That is not a product bug, but it will make the 1.0 loop
unnecessarily expensive unless we fix it early.

## 2026-04-09 15:40 - `.dockerignore` was worth doing immediately

Adding a minimal `.dockerignore` was low risk and paid off quickly. The Docker
build still succeeds, but the context dropped from multi-GB to roughly 128 MB.
That means the heavy validation loop is no longer blocked on avoidable context
transfer overhead before the real 1.0 config work even starts.

## 2026-04-09 17:10 - Config lived cleanly once policy stayed typed

The remaining product gap was mostly not UI work. The clean path was to keep
policy typed at the config/runtime seam and then thread only the resulting
values into state and services. That kept the reducer pure while still letting
runtime behavior change in ways the top-level spec promised.

The main useful split was:
- startup config chooses notification cooldown, startup profile, backend order,
  and Claude mode preference
- `AppState` owns only the notification values the reducer needs to reconcile
  transitions
- runtime owns backend construction and integration-source preference

## 2026-04-09 17:25 - Notification E2E needed deterministic shell resolution

The new runtime notification E2E initially failed because the backend wrapper
used `sh -lc`, which let shell startup behavior interfere with backend lookup
order on this machine. Switching the wrapper to `sh -c` made backend
resolution deterministic and is a better fit for non-interactive notification
dispatch anyway.

The other useful hardening step was atomic native-signal updates in the E2E
tests. Writing JSON in place can race the runtime refresh loop and create a
false compatibility fallback on one poll.

## 2026-04-09 17:55 - Plan retrospective

What matched the plan:
- the remaining 1.0 work really was a contract-completion pass centered on
  config, logging, and preference wiring rather than new TUI surfaces
- each gap closed cleanly as a vertical slice with unit coverage plus focused
  tmux-backed E2E tests
- the heavy validation loop stayed green once Colima and Docker context were
  healthy

What diverged:
- notification backend resolution needed one runtime hardening change
  (`sh -c` instead of `sh -lc`) that was not called out up front
- the runtime notification E2E also needed atomic native-signal updates to make
  the transition tests stable

What would enable more one-shot execution next time:
- encode the non-interactive shell-wrapper rule in architecture or a skill
- keep config-owned runtime policy grouped into small context structs early, so
  `clippy::too_many_arguments` never becomes cleanup work at the end

## 2026-04-10 09:40 - Final spec sync needed one more support-matrix pass

The biggest remaining drift was not a missing feature. It was that the top-level
spec and README made the native-heavy path very visible, but they did not spell
out the whole shipped support matrix as directly as they could. Adding the
matrix made it clear that Claude, Codex, and Pi have native overlays today,
while Gemini CLI and OpenCode are still compatibility-only.

The other repo-level drift was smaller but worth fixing: several plan
`META.yaml` files used `completed`, while the repo's documented lifecycle uses
`complete`. That kind of metadata mismatch does not break the product, but it
does weaken the plan contract the repo is trying to keep deterministic.

The same pass also surfaced a validation wrinkle that was worth fixing in repo
code rather than hand-waving away as local state. `mise run verify` only needs
to build a public Docker image here, but Docker was inheriting stale `gcr`
credential helpers from the user's global config and emitting noisy auth
warnings before the build succeeded. Moving the build step onto a temporary
clean `DOCKER_CONFIG` while preserving the active Docker host made the heavy
gate deterministic again.
