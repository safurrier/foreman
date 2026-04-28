# Changelog

## 1.1.1 - 2026-04-28

- Improve macOS notifications with `alerter` actions that can focus the
  relevant tmux window and pane after a notification click.
- Add configurable notification sound profiles, including audio-file and
  directory-backed completion and needs-attention sounds.
- Shorten desktop notification copy for notification surfaces and document the
  macOS VoiceOver action-menu path.
- Add notification diagnostics for `alerter`, sound playback, and tmux focus
  commands, and fall back to later notification backends when `alerter` exits
  early with an error.

## 1.1.0 - 2026-04-27

- Release operator polish across popup startup cache reuse, stable default
  sorting, runtime UI preference persistence, and resettable UI state.
- Add setup and diagnosis improvements, including `foreman --config-show`,
  broader `foreman --doctor` readouts, safe repair help, and setup hints when
  native hook fallback looks suspicious.
- Ship native status support for Claude, Codex, and Pi hook signals with
  native-over-compatibility precedence and stricter real-harness validation.
- Rework the dashboard information hierarchy with quieter singleton rows,
  stable Details sections, labeled footer action groups, inline search, and
  scrollable help/legend behavior.
- Move slow pull request lookup work off the UI loop, preserve drafts across
  failed tmux side effects, and harden search, flash, focus, and popup
  notification behavior.
- Refresh release and UX validation evidence for the `1.1.0` release lane.

## 1.0.0 - 2026-04-10

- Ship the interactive Ratatui dashboard for multi-session tmux monitoring,
  focus, direct input, search, flash navigation, pull request awareness, and
  notifications.
- Ship native integrations for Claude Code, Codex CLI, and Pi, plus
  compatibility integrations for Gemini CLI and OpenCode.
- Ship real tmux-backed smoke coverage for the main operator flows and opt-in
  real-binary E2E coverage for Claude, Codex, and Pi native integrations.
- Ship GitHub Actions release automation that verifies the repo, builds release
  bundles, and publishes tagged artifacts containing `foreman` and the hook
  bridge companion binaries.
