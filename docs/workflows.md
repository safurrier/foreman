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

Native readiness preflight:

```bash
mise run native-preflight
```

Opt-in real harness lane:

```bash
mise run verify-native
```

Local install or compatibility fallback diagnosis:

```bash
foreman --doctor
foreman --setup --user --project --dry-run
foreman --setup --user --project
```

Setup workflow notes:
- Treat `foreman --setup` as the supported install and repair command, not a one-off bootstrap.
- `--user` wires the home-level Claude, Codex, and Pi integration files.
- `--project` wires one repo at a time, either the current checkout or the repo passed with `--repo`.
- If you need another checkout, rerun setup for that repo:
  `foreman --setup --project --repo /path/to/repo`
- If you only want one provider, use `--claude`, `--codex`, or `--pi`.
- Setup is safe to rerun. It should converge files instead of creating drift.
- Setup does not fix already-running panes. Restart the affected agent panes after changing hook wiring.

Use `--repo /path/to/repo` when you need to diagnose or set up a different
checkout. `--setup` is intentionally conservative. It can initialize Foreman
config, merge Claude and Codex hook wiring, scaffold the Pi extension, and you
can scope it with `--user`, `--project`, `--claude`, `--codex`, and `--pi`.

Strict closeout for native harness work:

```bash
mise run native-preflight
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

Skip-only `mise run verify-native` runs do not satisfy done for slices that
change real harness behavior.

If `vhs` is installed, `mise run verify` now refreshes the UX GIF and PNG
artifacts through `mise run verify-ux --capture-only` after the heavier Rust
checks pass.

`mise run verify` now also runs `mise run verify-release`, which reruns the
compiled-binary release gauntlet serially and writes a report under:

`.ai/validation/release/`

That rerun is intentional. `cargo test --all-features` proves the broader Rust
suite, and `verify-release` reruns the operator gauntlet as the explicit
report-producing release proof.

GitHub Actions now uploads the same evidence as review artifacts:

- `validation-evidence` on pull requests from `.github/workflows/ci.yml`
- `release-validation-evidence` on tag verification from `.github/workflows/release.yml`

Those artifacts include:

- `.ai/validation/ux/`
- `.ai/validation/release/`

In CI and release runs, those validation artifacts are required. Missing UX or
release evidence is a real failure, not a skipped upload.

`mise run verify-ux` also runs the ignored `runtime_profiling` smokes so lag
regressions caused by synchronous lookup work or repeated `j/k` navigation under
load show up in the heavy lane instead of only in manual UX testing. That lane
now also checks staged tmux refresh behavior so full-inventory capture regressions
show up before they reach manual tmux testing.

The runtime smoke in that lane now covers the live help/legend surface,
small-layout help scrolling, harness-view cycling, deterministic native-hook
provenance and attention surfacing in the live dashboard, live popup focus with
`f`, and acting on the filtered selection inside a real tmux server.

For local lag triage, `foreman --debug` now emits timing lines for:
- `action` with enriched `move-selection` context
- `render_frame`
- `inventory_tmux`
- `inventory_native`

When `inventory_tmux` looks expensive, check whether the line shows high
`captures=` counts or whether cached preview reuse has dropped unexpectedly.

Those logs land in the active `latest.log` file under the configured Foreman log
directory and are the first place to check before chasing tmux itself.

That split is intentional:
- `mise run verify` proves the dashboard/operator surface inside a real tmux
  server without relying on provider auth or network behavior.
- `mise run verify-native` proves the actual Claude, Codex, and Pi CLI contracts
  in isolated temp environments.

The release gauntlet expands that proof into three coherent compiled-binary
journeys:
- startup, discovery, theme, help, and harness filtering
- startup, discovery, theme, scrollable help, and harness filtering
- compose, focus, search, flash, sort, rename, spawn, and kill
- PR state, notification policy, graceful degradation, and operator-alert logging

Opt-in real harness commands:

```bash
FOREMAN_REAL_CLAUDE_E2E=1 FOREMAN_REAL_CODEX_E2E=1 FOREMAN_REAL_PI_E2E=1 mise run verify-native

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
- The real Codex hook lane now requires a Codex CLI on PATH that exposes the
  `codex_hooks` feature and supports `UserPromptSubmit`. Old shadow installs can
  fail even when a newer Codex exists elsewhere on disk, so `mise run
  native-preflight` checks the PATH-resolved binary before the real suite runs.
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
