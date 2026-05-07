---
id: macos-overlay-unvibecode-architecture-spec
title: Spec — macOS Overlay Un-vibecode Architecture Pass
---

# Spec — macOS Overlay Un-vibecode Architecture Pass

## Goal

The native Foreman macOS overlay is now functionally close to the desired product.
This pass preserves behavior while deepening shallow modules one slice at a time.

Use the architecture vocabulary from `mattpocock-skills-improve-codebase-architecture`:

- **Module** — anything with an interface and an implementation.
- **Interface** — everything a caller must know to use the module correctly.
- **Implementation** — the code inside.
- **Depth** — leverage at the interface.
- **Seam** — where an interface lives.
- **Adapter** — a concrete thing satisfying an interface at a seam.
- **Leverage** — what callers get from depth.
- **Locality** — what maintainers get from depth.

## Non-goals

- Do not add broad product features during this pass.
- Do not redesign the overlay UX except where needed to preserve existing behavior
  through a deeper module.
- Do not combine slices. Each slice should leave the repository validating before
  moving to the next one.

## Baseline behavior to preserve

- `Foreman.app` launches and shows overlay.
- `Ctrl+F` is the default user shortcut.
- Agent list/search/filter/sort works.
- Smart row titles disambiguate duplicate workspaces.
- List/PR/Details/Compose focus regions work.
- Enter in PR opens browser; Enter in Details focuses pane.
- Compose sends text.
- Focus hides Foreman and foregrounds terminal adapter.
- Settings opens above overlay and Esc closes Settings only.
- Headless snapshot/OCR, fake gauntlet, real tmux smoke, and app bundle smoke pass.

## Slice 1 — Settings UI Module

**Problem:** real Settings lives in executable `main.swift`; snapshot target renders a duplicate `SettingsSnapshotView`.

**Solution:** move the real Settings view into `ForemanOverlayUI` so app and snapshot use one implementation.

**Validation:** settings snapshots render real Settings; Swift tests and full overlay verifier pass.

## Slice 2 — Split executable app shell

**Problem:** `ForemanOverlay/main.swift` is a shallow mega-module containing panel lifecycle, menus, hotkeys, terminal activation, settings window, app lifecycle, env config, and gauntlet hooks.

**Solution:** split into app-shell modules with boring interfaces: `AppDelegate`, `OverlayPanelController`, `AppMenus`, `SettingsWindowController`, hotkey, terminal, keyboard input, gauntlet hooks.

**Validation:** no behavior changes; full verifier passes.

## Slice 3 — Hotkey diagnostics module

**Problem:** persisted `KeyboardShortcuts` and env Carbon override are separate paths with little observability. `Ctrl+F` has been flaky for the user.

**Solution:** deepen hotkey handling with status/diagnostics: current label, source, registration path, env override, reset behavior, and app/status label updates.

**Validation:** unit tests for default/env parsing; app smoke; manual `Ctrl+F` check if possible.

## Slice 4 — Keyboard event adapter

**Problem:** key mapping is split between panel key equivalents and local event monitor, while Core already has `OverlayKeyboardCommand`/`OverlayKeyboardEffect`.

**Solution:** route AppKit key input through one adapter that emits `OverlayKeyboardCommand` and executes effects consistently.

**Validation:** reducer tests remain green; UI gauntlet passes; help/footer copy stays accurate.

## Slice 5 — Terminal activation module

**Problem:** terminal activation has a protocol but lives in executable code and stores raw preference strings.

**Solution:** extract terminal activation preferences, factory, and adapters into a testable module.

**Validation:** parsing tests for `auto`, `none`, known terminals, `bundle:<id>`, custom bundle id; gauntlet terminal skip remains non-fatal.

## Slice 6 — Snapshot registry single source

**Problem:** snapshot states are duplicated across renderer, scripts, OCR expectations, and docs.

**Solution:** make the snapshot renderer able to list states/metadata and scripts consume that where practical.

**Validation:** rendering and OCR cover all states; adding a state has one registry edit plus OCR expectation only when semantic checks are needed.

## Slice 7 — Split overlay view modules

**Problem:** `OverlayView.swift` is a large shallow UI module with root view, list, row, details, PR card, compose, footer, help, theme helpers, and some app side effects.

**Solution:** split into focused view modules while preserving the same store interface initially.

**Validation:** snapshots are primary proof; especially attention, pr-active, compose, help, duplicate-workspace.

## Slice 8 — Narrow OverlayStore

**Problem:** `OverlayStore` is deep for state/reducer behavior but also owns async process actions and app callbacks.

**Solution:** preserve `OverlayStore` as state + reducer, and move app/process side-effect execution behind a narrower seam where safe.

**Validation:** unit tests assert effects; process/app behavior stays covered by fake runner and gauntlet.
