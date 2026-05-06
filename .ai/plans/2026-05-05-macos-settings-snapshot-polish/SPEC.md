---
id: macos-settings-snapshot-polish-spec
title: Spec — macOS Settings Snapshot and Polish
---

# Specification — macOS Settings Snapshot and Polish

## Problem

Settings is now functional and less ugly, but still needs a proper visual
validation loop and a few UX fixes:

- No deterministic Settings snapshot exists, so Settings visual regressions are
  manual-only.
- Popup size sliders feel too low-level; preset sizes are better for a command
  palette app.
- Settings does not respect the selected Foreman overlay theme.
- Escape behavior should close Settings when Settings is frontmost, without
  quitting/hiding all of Foreman.
- Default shortcut should be Ctrl+F if it does not conflict with standard macOS
  expectations.

## Requirements

### Settings snapshots

- Add a headless Settings renderer executable or extend the existing snapshot
  executable to render Settings states.
- Render at least:
  - General
  - View
  - Focus
  - Notifications
- Write to:

```text
.ai/validation/macos-overlay/headless-snapshots/settings-*.png
```

- Include OCR semantic expectations for Settings snapshots.
- Include Settings snapshots in `mise run render-macos-overlay-snapshots` and
  `mise run verify-macos-overlay`.

### Popup size presets

Replace width/height sliders with size presets:

- Compact
- Default
- Large
- Extra Large
- Custom only if needed later

Suggested dimensions:

| Preset | Width | Height |
|---|---:|---:|
| Compact | 760 | 500 |
| Default | 900 | 600 |
| Large | 1100 | 720 |
| Extra Large | 1280 | 840 |

Settings should show the selected preset and dimensions.

### Theme-aware Settings

- Settings should use the same background/accent treatment as the overlay where
  practical.
- At minimum, it should avoid raw default Form/window styling that clashes with
  overlay themes.
- Theme can remain a lightweight visual treatment; full theme editor is not
  required.

### Escape behavior

- Pressing Esc while Settings is key should close Settings.
- Esc should not terminate Foreman.
- Esc should not close the main overlay unless the overlay is key/frontmost.

### Default shortcut

- Change default shortcut to Ctrl+F unless validation/manual check finds it is a
  problematic standard shortcut.
- Existing user override should not be overwritten unless no shortcut is stored.
- Reset Shortcut should reset to Ctrl+F.

## Acceptance

- Settings snapshots render in headless validation.
- OCR checks include Settings states.
- Settings size UI uses presets, not sliders.
- Settings appearance is visually aligned with the current overlay theme.
- Esc closes Settings alone.
- New installs/reset default shortcut is Ctrl+F.
- `mise run verify-macos-overlay` passes.
