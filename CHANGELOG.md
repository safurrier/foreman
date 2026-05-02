# Changelog

## 1.2.0 - 2026-05-02

### Added

- Added macOS custom notification sound prefixes with `notification-sounds:<prefix>`.
  Matching AIFF/AIF/CAF/WAV files in `~/Library/Sounds` now play through
  `alerter --sound`, so custom sounds stay on the macOS notification path and
  can respect Focus / Do Not Disturb.
- Added runtime identity checks for native harness detection so Foreman can
  distinguish Claude, Codex, and Pi panes more reliably when stale terminal
  scrollback mentions another tool.
- Added simulated tmux E2E coverage for native warning behavior, native status
  purity, and compatibility-mislabel correction.

### Changed

- Kept native provider status hook-only. Foreman no longer promotes native
  `working` signals to attention based on heuristic terminal text.
- Improved native overlay behavior so valid native signals can correct
  compatibility-only harness mislabels when runtime identity is unavailable,
  while still preventing wrong-provider native files from overriding panes with
  known runtime identity.
- Improved native warning diagnostics to focus warnings on real missing native
  signals instead of panes that were misclassified by compatibility heuristics.
- Advanced random sound selection salt on each resolution so random sound
  profiles avoid artificially sticky choices.

### Fixed

- Fixed false positive "needs attention" states caused by stale question,
  confirmation, or waiting-for-input text inside Codex and Pi panes.
- Fixed false Claude native warning counts for visible Codex/Pi panes that had
  reliable native or runtime identity evidence.

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
