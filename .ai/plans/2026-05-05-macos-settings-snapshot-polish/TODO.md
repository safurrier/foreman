---
id: macos-settings-snapshot-polish-todo
title: TODO — macOS Settings Snapshot and Polish
---

# TODO

## Phase 1 — Shortcut default

- [ ] Check Ctrl+F conflict risk.
- [ ] Change `KeyboardShortcuts.Name.toggleForemanOverlay` default to Ctrl+F.
- [ ] Change reset button copy/behavior to Ctrl+F.
- [ ] Update docs/help/settings copy.
- [ ] Do not overwrite existing user-stored shortcuts except reset/no-shortcut.

## Phase 2 — Size presets

- [ ] Add `OverlaySizePreset` enum in Core.
- [ ] Persist selected size preset.
- [ ] Replace Settings width/height sliders with preset picker/ticker.
- [ ] Apply selected preset to panel size.
- [ ] Keep bounds for direct resize, if direct resize remains enabled.
- [ ] Decide whether direct resize sets preset to custom or persists dimensions separately.

## Phase 3 — Theme-aware Settings

- [ ] Pass current theme / store / preferences to Settings view.
- [ ] Apply matching material/background/accent to Settings.
- [ ] Avoid raw Form layout and raw system gray cards that clash with overlay theme.
- [ ] Verify System/Indigo/Terminal themes look acceptable.

## Phase 4 — Esc closes Settings only

- [ ] Add `SettingsPanel` subclass or local key monitor for Esc.
- [ ] Ensure Esc closes Settings when Settings is key.
- [ ] Ensure Esc still closes overlay when overlay is key.
- [ ] Add a small scripted or unit proof if practical.

## Phase 5 — Settings snapshots

- [ ] Extend `foreman-overlay-snapshot` or add `foreman-settings-snapshot`.
- [ ] Render Settings states:
  - [ ] settings-general
  - [ ] settings-view
  - [ ] settings-focus
  - [ ] settings-notifications
- [ ] Add OCR expectations.
- [ ] Include in render/verify scripts.
- [ ] Inspect snapshots manually.

## Phase 6 — Validation

- [ ] `swift test --package-path apps/macos-overlay`
- [ ] `mise run render-macos-overlay-snapshots`
- [ ] Inspect settings PNGs.
- [ ] `mise run verify-macos-overlay`
- [ ] `mise run install-macos-overlay-app`
- [ ] Manual: Esc closes Settings only.
- [ ] Manual: Reset Shortcut sets Ctrl+F.
