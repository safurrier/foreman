# macOS Overlay Follow-up Hardening

## Goal

Fix the remaining A+ blockers found after the previous cleanup pass, with the highest priority on keyboard mode routing and the still-unreliable `Ctrl+F` hotkey.

## Slices

1. **Global hotkey reliability** — align Carbon registration with the event dispatcher target and keep diagnostics truthful.
2. **Keyboard mode pass-through** — flash/help modes must win over focused search text editing, while normal search/compose editing stays native.
3. **UI gauntlet coverage** — scripted gauntlet must prove flash label selection, `?` help, and help scrolling.
4. **Terminal activation lifecycle** — preserve the stateful activator instead of recreating auto activation on focus.
5. **Settings hotkey status freshness** — keep Settings hotkey status live after reset/recorder changes without reconfiguring on unrelated defaults.
6. **includeAllPanes reload semantics** — changing the setting should trigger a debounced reload because it changes the CLI query.

## Validation

Each slice must run:

```bash
swift test --package-path apps/macos-overlay
mise run validate-macos-overlay-change
```

Final completion must install and relaunch:

```bash
mise run install-macos-overlay-app
```
