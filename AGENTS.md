# foreman

**When a user corrects you or gives repo-specific tribal knowledge, document it
in the closest `AGENTS.md` before continuing.**

Foreman is a Rust/Ratatui operator console for AI agents running in tmux. Treat
`SPEC.md` as the product contract, `docs/architecture.md` as the architecture
record, and `docs/workflows.md` as the validation/process guide.

## How to Work Here

For meaningful work, create a feature branch and use the Harness Kit lifecycle:
`hk start <slug> --plan "..." --target .`, `hk validate --check <name> --why
"..." -- <command>`, `hk sync --target .`, and `hk ready --target .`. Export a
compact handoff package to `.ai/hk/<work-id>/` for PR-sized work. Historical
`.ai/plans/**` directories remain useful archaeology, but new work should not
create them by default. Use the smallest validation layer that proves the slice,
then run `mise run check` before push and `mise run verify` before merge or
runtime/release-sensitive changes.

## Commands

**Setup**: `mise run setup`.

**Fast gate**: `mise run check`.

**Heavy gate**: `mise run verify`.

**Release confidence**: `mise run verify-release`.

**Native harness proof**: run `mise run native-preflight`, then require the real
Claude, Codex, and Pi E2Es with `mise run verify-native`.

**macOS overlay changes**: `mise run validate-macos-overlay-change`.

**Local app**: `mise run dev`.

## Gotchas

- **DO** use portable `sh` in tmux smoke tests. **NOT** `zsh`. **BECAUSE**
  GitHub Linux runners do not guarantee `zsh`, and panes can exit immediately.

- **DO** write native-signal fixtures atomically. **NOT** overwrite them
  in-place. **BECAUSE** partial reads create false compatibility fallback and
  flaky runtime tests.

- **DO** promote recurring workflow lessons out of `.ai/plans/*` or `.ai/hk/*`
  into `docs/` or `AGENTS.md`. **NOT** treat historical plan logs or generated
  HK exports as canonical truth. **BECAUSE** lifecycle artifacts are evidence
  for a slice, not long-term onboarding.

- **DO** use HK for new meaningful work and commit compact `.ai/hk/<work-id>/`
  exports when a handoff package is useful. **NOT** hand-author new
  `.ai/plans/**` directories unless maintaining legacy workflow evidence.
  **BECAUSE** HK is now the lifecycle source of truth while `.ai/plans/**` is
  historical.

- **DO** commit structured `.ai/hk/`, legacy `.ai/plans/`, and `.ai/validation/`
  paths that the workflow depends on. **NOT** commit `.ai/handoffs/`,
  `.ai/research/`, HK artifact payloads, or plan-local artifact scratch unless
  explicitly needed for review. **BECAUSE** only compact handoff exports and
  stable validation roots are durable repo context.

- **DO** check the active Docker context when `mise run verify` fails in the
  Docker phase. **NOT** assume the Rust app regressed first. **BECAUSE** the
  common failure mode here has been local Colima or Docker runtime state.

- **DO** treat strict native verification as part of done when touching real
  harness or hook behavior. **NOT** count skip-only `mise run verify-native`
  runs as done. **BECAUSE** the real-provider E2Es are the only proof that
  native Claude, Codex, and Pi wiring still works end to end.

- **DO** keep native integration status pure to provider hook/file signals.
  **NOT** promote native panes with terminal text heuristics. **BECAUSE**
  heuristics are intentionally compatibility behavior; mixing them into native
  provenance makes Foreman look precise while it is guessing.

- **DO** make double-clicking a macOS overlay agent row focus that pane.
  **NOT** require the footer Focus button for pointer-driven selection.
  **BECAUSE** double-click-to-open/focus is expected macOS list behavior for
  this overlay.

- **DO** verify remote SSH sources use the same remote tmux binary/path as the
  attached terminal session, especially on Coder. **NOT** assume `/usr/bin/tmux`
  can query a server created by Homebrew/Nix tmux. **BECAUSE** Coder's
  non-login SSH path can pick `/usr/bin/tmux 3.2a`, while the interactive
  `tmux -L user` server may be `tmux 3.6a`; the symptom is `server exited
  unexpectedly` or missing remote agent panes until the source command loads
  the login PATH or uses a wrapper.

- **DO** treat normal typing in the macOS overlay as search input and arrow
  keys as agent navigation. **NOT** require users to manually re-focus the
  search field after clicking around. **BECAUSE** the overlay should behave like
  a command palette: open, type, move selection, act.

- **DO** call the macOS app **Foreman** in bundle names, menus, and user-facing
  copy. **NOT** ship user-facing **Foreman Overlay** naming. **BECAUSE** the
  native app is the Mac entry point for Foreman, not a separate product.

- **DO** keep overlay chrome out of content flow: active-region indicators,
  compose controls, and help overlays must not cover rows or text fields.
  **NOT** badge over selected rows or input controls. **BECAUSE** screenshots
  showed the List/Compose badges obscuring content.

- **DO** make modal/help overlays trap pointer scrolling and keyboard handling.
  **NOT** let help scrolling also scroll the underlying agent list. **BECAUSE**
  foreground overlay interactions should not mutate background state.

- **DO** back menu-equivalent hotkeys such as Cmd+T with real app/menu commands
  or an AppKit key-equivalent handler. **NOT** rely only on SwiftUI/local event
  monitors for Command shortcuts. **BECAUSE** macOS may play the invalid-command
  sound even when the local monitor changes state.

- **DO** make Settings visibly open above the overlay. **NOT** allow the settings
  panel to appear behind the floating overlay. **BECAUSE** users read that as
  Settings not opening.

- **DO** restore the macOS popup when the user re-activates the Foreman app via
  Cmd+Tab, Spotlight/Raycast, Dock, or app switcher. **NOT** require only the
  global hotkey after the panel hides. **BECAUSE** a regular Mac app should be
  recoverable through normal app activation flows.

- **DO** return focus to the previous non-launcher app when Esc hides the macOS
  overlay. **NOT** leave Foreman as the active app with no visible panel.
  **BECAUSE** command-palette overlays should disappear back to the user's
  working context.

- **DO** treat Attention → Recent as attention rank followed by actual tmux pane
  activity recency. **NOT** rely only on native status activity scores once panes
  become Idle. **BECAUSE** idle native signals often share the same score, while
  users expect recently active sessions to stay above older idle sessions.

- **DO** preserve type-to-search in the macOS overlay when adding TUI parity
  shortcuts like flash jump. **NOT** steal plain `s` for flash navigation unless
  an explicit mode/setting makes that tradeoff clear. **BECAUSE** the overlay's
  command-palette interaction model depends on plain typing going to search.

- **DO** let AppKit text fields own normal search/compose editing when focused.
  **NOT** manually mutate text for cursor movement, selected-text replacement,
  Option+Delete, or Cmd+A paths. **BECAUSE** the overlay should feel like a
  native Mac command palette, not a custom terminal prompt.

- **DO** use the `KeyboardShortcuts` package for both persisted macOS overlay
  shortcut recording and normal global shortcut handling. **NOT** register a
  second custom Carbon hotkey for the same persisted shortcut. **BECAUSE** mixed
  ownership races the recorder and can leave Settings showing stale registration
  failures.

- **DO** run `mise run validate-macos-overlay-change` for Swift overlay,
  app-bundle, keyboard/focus, screenshot, or control-API changes. **NOT** treat
  plain `swift test` as sufficient for these paths. **BECAUSE** the required
  lane also proves fake-Foreman UI events, real tmux smoke, snapshots/OCR, and
  app bundle launch.

- **DO** preserve direct-argv tmux popup bindings like
  `display-popup -h 80% -w 80% -E -- "$HOME/.cargo/bin/foreman" --popup`.
  **NOT** wrap the popup command in a shell unless expansion is required.
  **BECAUSE** shell startup, especially zsh init, can add seconds of latency to
  Foreman's quick popup path.

- **DO** keep the tmux popup and macOS overlay conceptually parallel as Foreman
  surfaces. **NOT** make cross-source visibility a Mac-only capability by
  default. **BECAUSE** divergent surfaces make the product harder to reason
  about; operators should not need to remember that "global overview" only
  exists in the native app.

- **DO** validate source companion reverse tunnels with a real JSON-line
  companion request. **NOT** probe readiness by opening and immediately closing
  the forwarded TCP port. **BECAUSE** a half-open/empty probe can consume or
  stall the companion server's first single-threaded request and make a working
  `ssh -R` tunnel look broken with `Connection refused` later.

- **DO** make all-source popup UX stay fast and readable with local-first,
  config-driven source labels, deduped session grouping, and async/cached remote
  refresh. **NOT** block every popup open or navigation move on live SSH, or
  force users to cycle source targets manually. **BECAUSE** Foreman is a
  command-palette-like operator console where users expect all work at a glance
  and Enter/focus to jump directly to the selected local or Coder pane.

- **DO** keep popup keypress performance covered by `mise run verify-ux` and
  `scripts/smoke-popup-key-latency.sh` when touching runtime scheduling,
  inventory refresh, source aggregation, PR/extension lookup cadence, or tmux
  capture. **NOT** rely on subjective manual popup testing. **BECAUSE** key-lag
  regressions have recurred, and the smoke enforces local-only, all-source idle,
  and refresh-overlap navigation budgets.

## Related Context

| Path | What's there |
|---|---|
| `docs/tour.md` | First read, repo map, and daily loop |
| `docs/workflows.md` | HK lifecycle, validation ladder, and environment notes |
| `docs/architecture.md` | System boundaries, invariants, and module map |
| `docs/macos-overlay/` | Swift macOS overlay architecture, app bundle/install notes, UX checklist, and validation ladder |
| `.agent/skills/foreman-swift-overlay-ux/` | Foreman-specific Swift overlay UX review workflow |
| `.agent/skills/foreman-swift-overlay-validation/` | Foreman-specific Swift overlay validation workflow |
| `.ai/hk/AGENTS.md` | Generated HK export rules |
| `.ai/plans/AGENTS.md` | Legacy plan artifact contract |
| `README.md` | Human quickstart, install, dashboard keys, and status matrix |

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-04-30 -->
