---
id: workflows
title: Workflow Guide
description: >
  Durable workflow and validation guidance for foreman, including plan hygiene,
  tmux E2E rules, harness-specific test notes, and Docker/CI rough edges.
index:
  - id: plan-artifacts
  - id: validation-ladder
  - id: tmux-e2e-rules
  - id: native-harness-notes
  - id: environment-and-release-notes
---

# Workflow Guide

## Plan Artifacts

For every meaningful slice, create a plan under `.ai/plans/` and keep these
files current while you work:

| File | What to keep current |
|---|---|
| `META.yaml` | `branch`, `pr`, and `status` |
| `TODO.md` | What is actually left |
| `LEARNING_LOG.md` | Surprises, failures, and design adjustments |
| `VALIDATION.md` | What was run and what passed or failed |

Use `status: complete`, not `completed`.

## Validation Ladder

Foreman works best when every slice names its dominant validation layer up
front, then adds at least one real-environment test when the slice crosses a
real external seam.

| Layer | Typical proof |
|---|---|
| Unit | Reducer, config, precedence, debounce |
| Render | Ratatui buffer tests |
| Adapter contract | Fake-backed tmux, notifications, PRs |
| Runtime smoke | Real tmux fixture, compiled `foreman` binary |
| Release gauntlet | Compiled-binary operator walkthroughs with temporary tmux worlds plus a durable checklist/report artifact |
| UX artifact refresh | `vhs` walkthrough and screenshots from the live binary across at least one branded palette and the no-color fallback |
| Navigation perf smoke | Real tmux run with a fake PR backend plus run-log timing assertions |
| Real harness E2E | Opt-in ignored tests with the actual external CLI |

Focused UX lane:

```bash
mise run verify-ux
```

Release-confidence lane:

```bash
mise run verify-release
```

If `vhs` is installed, `mise run verify` now refreshes the UX GIF and PNG
artifacts through `mise run verify-ux --capture-only` after the heavier Rust
checks pass.

`mise run verify` now also runs `mise run verify-release`, which reruns the
compiled-binary release gauntlet serially and writes a report under:

`.ai/plans/2026-04-10-151735-release-validation-gauntlet/artifacts/`

`mise run verify-ux` also runs the ignored `runtime_profiling` smoke so lag
regressions caused by synchronous lookup work show up in the heavy lane instead
of only in manual UX testing.

The runtime smoke in that lane now covers the live help/legend surface plus
harness-view cycling and acting on the filtered selection inside a real tmux
server.

The release gauntlet expands that proof into three coherent compiled-binary
journeys:
- startup, discovery, theme, help, and harness filtering
- compose, focus, search, flash, sort, rename, spawn, and kill
- PR state, notification policy, graceful degradation, and operator-alert logging

Opt-in real harness commands:

```bash
FOREMAN_REAL_CLAUDE_E2E=1 cargo test --test claude_real_e2e -- --ignored --nocapture
FOREMAN_REAL_CODEX_E2E=1 cargo test --test codex_real_e2e -- --ignored --nocapture
FOREMAN_REAL_PI_E2E=1 cargo test --test pi_real_e2e -- --ignored --nocapture
```

## tmux E2E Rules

- Use `sh` in tmux fixture commands. Do not assume `zsh`.
- If a dashboard smoke test sends input or focuses a pane, make sure the target
  pane is recognized as an agent by the harness-detection layer.
- If tmux CI fails with `server exited unexpectedly` or `no server running` at
  the first session bootstrap, inspect fixture portability and startup timing
  before changing product code. Foreman's shared fixture already retries those
  transient startup errors.
- Keep alternate-screen capture fallbacks in mind. Detached tmux panes can
  return a successful but blank alternate-screen capture.
- Write native-signal fixture files atomically so refresh polls never see a
  half-written state.

## Native Harness Notes

- Native bridges stay outside the reducer. Claude and Codex use hook bridges;
  Pi uses a thin extension that calls `foreman-pi-hook`.
- Native-over-compatibility precedence belongs in one overlay layer, not inside
  the tmux adapter.
- Codex has one special validation wrinkle: local testing showed reliable hook
  emission at the non-interactive hook boundary, but not from the same command
  inside a tmux TTY. Keep the strongest real Codex proof at the hook boundary,
  and let Foreman's own tmux/runtime smoke tests cover the dashboard path.
- Gemini CLI and OpenCode are compatibility-only today.

## Environment and Release Notes

- `mise run verify` follows the active Docker context. If the context points at
  Colima, make sure Colima is running before debugging the app itself.
- `.dockerignore` and `Cargo.docker.toml` are part of the heavy validation path.
  They keep Docker context size and dependency-layer churn under control.
- The release pipeline must ship all companion binaries:
  `foreman`, `foreman-claude-hook`, `foreman-codex-hook`, and `foreman-pi-hook`.
- Historical `.ai` entries are evidence. Stable guidance belongs here, in
  [`tour.md`](tour.md), or in the root [`../AGENTS.md`](../AGENTS.md).
