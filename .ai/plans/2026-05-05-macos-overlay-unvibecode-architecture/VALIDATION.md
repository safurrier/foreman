---
id: macos-overlay-unvibecode-architecture-validation
title: Validation — macOS Overlay Un-vibecode Architecture Pass
---

# Validation

## Baseline checkpoint

Before refactoring:

```bash
mise run verify-macos-overlay
mise run install-macos-overlay-app
```

Latest known-good checkpoint before this plan:

- Swift tests: passed
- Fake UI gauntlet: passed
- Real tmux smoke: passed
- Headless snapshots/OCR: 16 states passed
- App bundle smoke: passed
- Installed app launched from `~/Applications/Foreman.app`

## Per-slice validation ladder

Use smallest useful layer first, then full gate before moving to the next slice.

### Narrow

```bash
swift test --package-path apps/macos-overlay
```

### Visual/snapshot

```bash
mise run render-macos-overlay-snapshots
```

### Full overlay gate

```bash
mise run verify-macos-overlay
```

### Install/relaunch after externally visible changes

```bash
mise run install-macos-overlay-app
open -a Foreman
```

## Slice log

| Slice | Narrow | Snapshot | Full | Install | Notes |
|---|---:|---:|---:|---:|---|
| 1 Settings UI Module | passed | passed | passed | passed | Real SettingsView moved into ForemanOverlayUI; snapshot target renders same view. |
| 2 Split executable app shell | passed | passed | passed | passed | Extracted panel, menus, settings panel, keyboard, hotkey, terminal, and gauntlet modules from main.swift. |
| 3 Hotkey diagnostics module | passed | passed | passed | passed | Hotkey registration status now logs and surfaces in status item tooltip/Settings. |
| 4 Keyboard event adapter | passed | passed | passed | passed | Panel and local monitor now share one AppKit-event to keyboard-command adapter. |
| 5 Terminal activation module | passed | passed | passed | passed | Terminal activation preference parsing moved into Core and covered by unit tests. |
| 6 Snapshot registry | passed | passed | passed | passed | Snapshot renderer lists states; render/verify scripts consume registry. |
| 7 Split overlay view modules | passed | passed | passed | passed | Extracted row/chip/style/help/PR/preview modules and removed direct AppKit hide from UI. |
| 8 Narrow OverlayStore | passed | passed | passed | passed | Replaced loose app callbacks with named OverlayAppRouting seam and router tests. |
