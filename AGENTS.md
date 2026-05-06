# foreman

**When a user corrects you or gives repo-specific tribal knowledge, document it
in the closest `AGENTS.md` before continuing.**

Foreman is a Rust/Ratatui operator console for AI agents running in tmux. Treat
`SPEC.md` as the product contract, `docs/architecture.md` as the architecture
record, and `docs/workflows.md` as the validation/process guide.

## How to Work Here

For meaningful work, create a feature branch, run `mise run plan -- <slug>`, and
keep the active plan current as scope changes. Use the smallest validation layer
that proves the slice, then run `mise run check` before push and `mise run
verify` before merge or runtime/release-sensitive changes.

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

- **DO** promote recurring workflow lessons out of `.ai/plans/*` into `docs/`
  or `AGENTS.md`. **NOT** treat historical plan logs as canonical truth.
  **BECAUSE** plan artifacts are evidence for a slice, not long-term onboarding.

- **DO** commit structured `.ai/plans/` and `.ai/validation/` paths that the
  workflow depends on. **NOT** commit `.ai/handoffs/`, `.ai/research/`, or
  plan-local artifact scratch. **BECAUSE** only the structured plan and
  validation roots are durable repo context.

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

- **DO** register the persisted macOS overlay shortcut through Foreman's
  `HotkeyController` and surface its Carbon registration status. **NOT** rely on
  `KeyboardShortcuts` handler state as proof the global hook works. **BECAUSE**
  the recorder is persistence/UI, while Carbon registration is the runtime seam.

- **DO** run `mise run validate-macos-overlay-change` for Swift overlay,
  app-bundle, keyboard/focus, screenshot, or control-API changes. **NOT** treat
  plain `swift test` as sufficient for these paths. **BECAUSE** the required
  lane also proves fake-Foreman UI events, real tmux smoke, snapshots/OCR, and
  app bundle launch.

## Related Context

| Path | What's there |
|---|---|
| `docs/tour.md` | First read, repo map, and daily loop |
| `docs/workflows.md` | Plan artifacts, validation ladder, and environment notes |
| `docs/architecture.md` | System boundaries, invariants, and module map |
| `docs/macos-overlay/` | Swift macOS overlay architecture, app bundle/install notes, UX checklist, and validation ladder |
| `.agent/skills/foreman-swift-overlay-ux/` | Foreman-specific Swift overlay UX review workflow |
| `.agent/skills/foreman-swift-overlay-validation/` | Foreman-specific Swift overlay validation workflow |
| `.ai/plans/AGENTS.md` | Plan artifact contract and lifecycle |
| `README.md` | Human quickstart, install, dashboard keys, and status matrix |

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-04-30 -->
