---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — doctor-install-ux

## Problem

- Local install currently has a sharp mismatch between what users think
  happened and what Foreman actually needs for native mode to work.
- `mise run install-local` installs the binaries, but it does not verify or
  scaffold the hook wiring that Claude, Codex, and Pi need.
- When native hooks are unwired, Foreman silently falls back to compatibility
  mode and the operator mostly learns that from logs or a vague
  `compatibility heuristic` label.
- Hook-command failures can appear as pane `error` states because compatibility
  heuristics only see pane text like `PreToolUse:Bash hook error`.
- The result is poor operator trust: the app looks broken even when the core
  problem is setup drift.

## Requirements

### MUST

- Add a reusable diagnosis engine that can inspect:
  - machine prerequisites
  - Foreman config/bootstrap paths
  - provider binary and hook-binary presence
  - repo hook wiring
  - runtime/native overlay summaries
  - recent hook-failure evidence where available
- Add a first-class CLI diagnosis surface with human-readable output.
- Support structured output via `--json` so tests and agents can consume it.
- Emit findings with:
  - severity
  - scope
  - evidence
  - likely cause
  - concrete next step
- Detect and explain the local failure modes already confirmed in investigation:
  - no config initialized
  - native directories missing or empty
  - known harness panes present but zero native signals applied
  - provider hook file missing
  - hook binary missing from `PATH`
  - Codex version/feature too old
  - hook-command failure text present in pane preview/logs
- Keep diagnosis separate from lifecycle truth. A setup problem should not need
  to masquerade as an agent failure to be visible.
- Add an in-product operator-facing diagnostic surface so the TUI can explain
  compatibility fallback and point to the fix path.
- Update install and first-run docs so native setup is framed as a second step,
  not implied by binary install alone.

### SHOULD

- Support repo-scoped diagnosis via an explicit path or current working tree.
- Provide safe auto-fix or scaffold flows for additive provider configs.
- Print exact files and snippets that would be written before any mutation.
- Point `mise run install-local` and `foreman --init-config` users toward
  diagnosis immediately after install.
- Add machine-readable finding IDs so logs, tests, and UI can refer to the same
  problems consistently.
- Distinguish setup degradation from ordinary compatibility fallback. Example:
  `no native signal observed` is different from `native intentionally disabled`.
- Reuse the same doctor engine inside runtime/bootstrap rather than duplicating
  checks in the TUI.

## Non-Goals

- Replacing native-preflight or the strict real E2E workflow
- Making Foreman responsible for provider authentication
- Auto-merging arbitrary user config files without narrow, tested merge rules
- Replacing the compatibility model; the goal is to explain and improve it

## Constraints

- Do not overwrite existing provider config blindly.
- Keep the CLI non-interactive by default. Prompts, if any, must be opt-in.
- Preserve current utility flag behavior unless a subcommand migration is
  clearly worth the churn.
- TUI diagnosis should stay compact and actionable, not become a second log
  viewer.
- Provider-specific checks must degrade cleanly when the provider CLI is not
  installed.

## Proposed UX contract

### CLI

- `foreman doctor`
  - runs machine + repo + runtime-adjacent checks available outside tmux
- `foreman doctor --repo /path/to/repo`
  - inspects provider hook files for that repo
- `foreman doctor --json`
  - emits structured findings
- `foreman doctor --strict`
  - exits non-zero when blocking findings exist
- `foreman doctor --fix`
  - only for safe, additive fixes

Human output should look like an operator report, not a raw dump:

```text
Machine
  OK   tmux 3.6a
  OK   foreman-codex-hook on PATH
  WARN foreman config not initialized at ~/.config/foreman/config.toml

Repo: /Users/alex.furrier/git_repositories/dots
  ERROR .codex/hooks.json missing
  WARN  .claude/settings.local.json exists but has no Foreman hook entries

Runtime hints
  WARN  5 Codex panes are falling back to compatibility and no native signals were observed
  INFO  pane preview includes "PreToolUse:Bash hook error"

Next steps
  foreman doctor --repo /Users/alex.furrier/git_repositories/dots --fix
```

### TUI

- When a pane is in compatibility mode, the context panel should say why if a
  known cause exists:
  - no native signal observed
  - hook file missing
  - hook command likely failing
  - native mode intentionally disabled
- Bootstrap/runtime should surface a compact operator alert when native
  integration is widely degraded, with a clear suggestion to run doctor.
- Hook-setup failures should become a diagnostic reason, not only a pane
  `error` inferred from preview text.

## Phased delivery

1. Doctor model and CLI reporting
2. Provider-specific repo checks and safe scaffold planning
3. Conservative `--fix` flows
4. Runtime/TUI diagnostics driven by doctor findings
5. Docs, screenshots, and full validation sync

## Acceptance criteria

- A fresh local install with missing hook wiring produces a clear diagnosis in
  one command.
- Doctor output makes it obvious whether the issue is machine, repo, or runtime
  state.
- Foreman can explain why a pane is in compatibility mode without forcing the
  operator to inspect logs.
- Safe fix flows never overwrite unrelated user config.
- Known local failure modes from this investigation are covered by tests.
