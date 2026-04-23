---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-13 13:05 - Local install investigation

- `foreman`, `foreman-claude-hook`, `foreman-codex-hook`, and `foreman-pi-hook`
  are installed and on `PATH`.
- `foreman --config-path` resolves to
  `/Users/alex.furrier/.config/foreman/config.toml`, but that file does not
  exist locally.
- `~/.local/state/foreman/logs/latest.log` exists, but
  `~/.local/state/foreman/{claude-native,codex-native,pi-native}` do not.
- Latest logs repeatedly show:
  - `claude_native_summary applied=0 fallback_to_compatibility=25 warnings=0`
  - `codex_native_summary applied=0 fallback_to_compatibility=5 warnings=0`
  - `pi_native_summary applied=0 fallback_to_compatibility=0 warnings=0`
- Conclusion: the app is installed and booting, but native hook data is not
  arriving at all. The screenshot's `compatibility heuristic` label is
  consistent with local state.

## 2026-04-13 13:12 - Hook wiring investigation

- Most affected repos do not contain the expected native-wiring files:
  - `.codex/hooks.json`
  - `.claude/settings.json`
  - `.pi/extensions/foreman.ts`
- Global config is also not wiring Foreman:
  - `~/.claude/settings.json` contains unrelated hooks
  - `~/.codex/hooks.json` is absent
  - `~/.codex/config.toml` has no hook bridge config
- Captured Codex pane output includes `PreToolUse:Bash hook error`.
- Current compatibility heuristics treat `error` and `failed` in pane previews
  as agent failures, so hook-command failures can masquerade as agent failures.
- Conclusion: install succeeds, but setup is incomplete and the operator gets
  weak feedback about why.

## 2026-04-13 13:20 - Product gap

- `mise run install-local` only installs binaries via `cargo install`.
- `foreman --init-config` only writes `config.toml`.
- `native-preflight` is aimed at real E2E readiness, not day-one operator setup.
- There is no reusable diagnosis engine, no repo-scoped install check, and no
  safe scaffold/fix path for missing hook files.
- The TUI explains status source, but not why a pane is in compatibility mode
  or how to get it out of compatibility mode.

## 2026-04-13 13:27 - Planned direction

- Add a reusable doctor engine that can inspect machine state, repo hook
  wiring, runtime/native overlay summaries, and likely hook failure signatures.
- Surface that engine in two places:
  - CLI: `foreman doctor`-style human and JSON output
  - Runtime/TUI: operator alert plus compact context hints
- Keep auto-fix conservative and idempotent. Codex and Pi are likely safe to
  scaffold first. Claude needs explicit merge rules before auto-write is safe.

## 2026-04-13 15:30 - Implementation closeout

- Kept the CLI as flat utility flags for v1:
  - `--doctor`
  - `--doctor-json`
  - `--doctor-strict`
  - `--doctor-fix`
  - `--doctor-dry-run`
  - `--doctor-repo`
- Added a shared `src/doctor.rs` engine instead of duplicating setup logic
  between CLI and runtime. The same findings now drive:
  - CLI diagnosis output
  - runtime diagnostics state
  - TUI context hints
- Safe auto-fix is now split by provider:
  - Foreman config init: safe and additive
  - Codex hooks: safe merge/create
  - Pi extension: safe scaffold
  - Claude settings: manual snippet only until merge rules are proven
- Runtime diagnosis is separate from lifecycle classification. Pane state still
  comes from lifecycle and compatibility signals, while setup drift appears as:
  - runtime diagnostics
  - diagnostics operator alerts
  - context-panel `Setup` guidance

## 2026-04-13 16:05 - Validation and UX notes

- `mise run verify` passed through all heavy phases:
  - fast gate
  - integration tests
  - Docker build
  - release gauntlet
  - UX/performance artifact refresh
- Strict native closeout passed with real harness lanes for Claude, Codex, and
  Pi by running `mise run verify-native` with
  `FOREMAN_REQUIRE_REAL_E2E=1`.
- A direct doctor dry-run surfaced one small UX gotcha during probing:
  `--doctor-dry-run` only makes sense with `--doctor-fix`. The CLI already
  enforces that correctly; the failed probe was user error in the command, not
  a docs mismatch.
- The doctor output on this machine is useful immediately. It reports the real
  cause behind the screenshot:
  missing config, missing native hook wiring, absent native signal dirs, and
  live hook-error previews that forced compatibility fallback.

## 2026-04-13 18:20 - CLI ergonomics follow-up

- The original doctor surface was still too hard to discover:
  - `--doctor --doctor-repo . --doctor-fix` was accurate but not a good first-run
    story
  - the docs were underselling that doctor already infers the current repo
- The better public contract is:
  - `foreman --setup`
  - `foreman --doctor`
  - `--repo /path/to/repo` only when targeting another checkout
- Implementation detail:
  - kept the old `--doctor-fix`, `--doctor-dry-run`, and `--doctor-repo` paths as
    hidden compatibility aliases
  - added a real `--setup` flow on top of the existing doctor engine instead of
    rewriting the backend model
- The help output matters as much as the command semantics here. Root `--help`
  is now responsible for teaching the first-time path without sending the user
  into README archaeology.

## 2026-04-15 13:45 - Scoped setup closeout

- `foreman --setup` needed a clearer mental model than “run setup.” The durable
  model is now:
  - `--user` for home-level wiring
  - `--project` for one repo at a time
  - `--claude` / `--codex` / `--pi` for narrowing writes
- Creating hook files was not enough. A “successful” setup that still left
  `~/.local/state/foreman/{claude-native,codex-native,pi-native}` missing made
  doctor immediately report native-dir drift. Setup now creates those native
  signal directories too.
- Running a real user-level setup changes the local machine enough to break
  tests that implicitly read `HOME`. The doctor and CLI tests now isolate
  `HOME` explicitly so they stay hermetic after local setup is used for real.
- The final local proof is narrower than “everything is green everywhere.” This
  repo is now installed and wired correctly, but existing panes in other repos
  still show compatibility fallback or hook errors until those repos are wired
  and the panes are restarted.
