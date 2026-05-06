---
id: macos-overlay-settings-filters-parity-validation
title: Validation — macOS Overlay Settings and Filters Parity
---

# Validation

## Automated proof — 2026-05-05

Ran:

```bash
swift test --package-path apps/macos-overlay
mise run render-macos-overlay-snapshots
mise run verify-macos-overlay
mise run install-macos-overlay-app
```

Results:

- Swift tests passed: 16 tests, including new preference/filter coverage.
- UI event gauntlet passed.
- Real tmux smoke passed.
- Headless screenshots rendered and OCR passed for 13 states.
- App bundle smoke passed.
- Installed and relaunched `~/Applications/Foreman.app`.

New test coverage:

- `testPreferencesClampAndPersistPopupSize`
- `testStoreAppliesPreferencesFiltersAndSort`
- existing all-panes client argument test still covers `--all-panes` wiring

Manual checks still recommended:

1. Open Settings → General and adjust popup width/height.
2. Close/reopen Foreman and confirm size applies.
3. Settings → View: toggle all panes, choose sort/filter values, refresh if all-panes changed.
4. Settings → Focus: choose terminal activation mode.
5. Confirm search bar text/cursor appears vertically centered on the live app.
6. Confirm Tab to Details remains visually obvious with the active border and footer region chip.
