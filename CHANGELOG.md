# Changelog

## 1.5.0 - 2026-06-10

### Added

- Added multi-source Foreman inventory with source-scoped pane identity for local
  and remote SSH-backed sources.
- Added source snapshots and prewarming so remote/source visibility can survive
  live companion outages with freshness diagnostics.
- Added the source companion JSON-line transport, including `foreman companion
  serve`, `foreman companion probe`, and live companion-backed sources.
- Added `foreman companion connect-ssh <host>` to start a local companion,
  open an SSH reverse tunnel, configure the remote source, and supervise the
  bridge.
- Added trusted reverse focus/send support for companion sources, gated by
  explicit send trust and companion tokens.
- Added source-local display activation for remote-triggered focus actions.

### Changed

- Updated the tmux popup and runtime refresh loop to keep all-source views
  responsive with local/cached rows first and idle-deferred remote merges.
- Updated the macOS overlay to decode source-aware control API fields, route
  remote focus/send actions through `--source`, and render source provenance.
- Updated source-companion validation to use isolated tmux sockets and Foreman's
  native `companion probe` instead of ad hoc remote socket probes.

### Fixed

- Fixed reverse-tunnel validation that could poison the companion server by
  opening and closing the forwarded TCP port without sending a request.
- Fixed remote source action routing so focus/send/extension lookups use
  source-scoped pane identity instead of bare tmux pane ids.
- Fixed send-capable companion setup so `--allow-send` requires token-backed
  trust.

## 1.4.0 - 2026-05-16

### Added

- Added a generic read-only extension provider platform for Foreman control API
  clients and the TUI Details pane.
- Added `foreman agents --json --extensions` for all-workspace extension card
  collection and `foreman extensions --pane <pane-id> --json` for selected-pane
  extension lookup.
- Added a Harness Kit provider example and docs that render HK lifecycle state as
  Foreman cards without mutating HK work state.
- Added explicit pane-to-repository links with `foreman links add/remove/list` so
  PR and provider lookups can target a code repo even when an agent runs from
  notes, scratch space, or a launcher directory.

### Changed

- Updated the macOS overlay to render PR/inventory first and fetch extension
  cards only for the selected pane in the background.
- Updated the README around the operator journey, control API, extension
  provider platform, and release process.
- Added a separate extension polling interval so local provider refresh cadence
  is not coupled to pull request polling.

### Fixed

- Preserved PR cards when slow extension providers time out in the macOS overlay.
- Avoided duplicate SwiftUI row identities for extension rows with the same label
  and value.
- Aligned extension status color buckets between the Rust TUI and Swift overlay.
- Avoided opening an unused stdin pipe when running provider commands without
  stdin.

## 1.3.1 - 2026-05-07

### Added

- Added `mise run pr-preflight`, a lightweight large-PR checklist task with
  cheap guardrails for staged local state, large screenshots, version metadata,
  and touched validation lanes.

### Changed

- Documented the `Attention → Recent` sort semantics for operators, including
  the real-recency tiebreaker for idle native sessions.
- Documented the post-merge release tag workflow and large-feature PR preflight
  checklist in the workflow guide.
- Ensured the macOS overlay verifier builds the overlay executable before the
  UI event gauntlet uses `FOREMAN_OVERLAY_SKIP_BUILD=1`.

## 1.3.0 - 2026-05-06

### Added

- Added a native macOS control app packaged as `Foreman.app`, installable to
  `~/Applications` and launchable through Spotlight, Raycast, Finder, `open -a
  Foreman`, a menu/status item, or a configurable global shortcut.
- Added a Rust control API for non-TUI clients:
  - `foreman agents --json`
  - `foreman agents --json --all-panes`
  - `foreman agents --json --pull-requests`
  - `foreman focus --pane <pane-id> --json`
  - `foreman send --pane <pane-id> --stdin --json`
  - `foreman send --pane <pane-id> --text <text> --json`
- Added macOS overlay UX for type-to-search, keyboard navigation, double-click
  focus, detail preview, compose/send, pull request cards/actions, flash jump,
  help, themes, filters, sort, Settings, and native text-editing preservation.
- Added deterministic macOS overlay validation with Swift unit tests,
  fake-Foreman UI gauntlet, real tmux smoke, headless snapshots/OCR, app-bundle
  smoke, and the `mise run validate-macos-overlay-change` validation lane.
- Added app bundle build/install/verify tasks, including generated app icon
  wiring from `foreman-logo.png`.

### Changed

- Updated `Attention → Recent` ordering so idle native sessions use real
  pane/native-signal recency instead of tying on static idle status scores.
- Hardened the macOS app install/reset flow so `mise run install-macos-overlay-app`
  unregisters/removes stale build and prototype bundles before registering only
  `~/Applications/Foreman.app`.
- Made app-bundle smoke validation non-activating so routine validation does not
  pop the overlay in front of the user's desktop.
- Updated docs, README, SPEC, and agent context for the macOS app, control API,
  validation lane, install workflow, and LaunchServices/Raycast/Spotlight gotchas.

### Fixed

- Fixed stale local app launches where Spotlight/Raycast could rediscover a
  repo-local dist app or old `Foreman Overlay.app` prototype after validation.
- Fixed macOS overlay lifecycle edges around Carbon hotkey registration,
  Settings/key-window routing, deterministic reload cancellation, stale response
  handling, selection normalization, subprocess cancellation, and PR lookup
  timeout fallback.
- Fixed overlay keyboard regressions so arrow navigation, help/flash shortcuts,
  comma search text, and AppKit text editing coexist correctly.
- Fixed Esc dismissal so hiding the overlay returns focus to the previous
  non-launcher app when possible, while pane focus still returns to the terminal.

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
