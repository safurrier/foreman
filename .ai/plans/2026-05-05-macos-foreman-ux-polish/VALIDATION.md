---
id: macos-foreman-ux-polish-validation
title: Validation — macOS Foreman UX Polish
---

# Validation

## Automated proof — 2026-05-05

Ran:

```bash
swift test --package-path apps/macos-overlay
mise run render-macos-overlay-snapshots
mise run verify-macos-overlay-app
mise run verify-macos-overlay
mise run install-macos-overlay-app
```

Results:

- Swift unit tests passed.
- UI-event XCTest gauntlet passed under `mise run verify-macos-overlay`.
- Real tmux control API smoke passed.
- Headless snapshots rendered for 12 states, including new `compose` state.
- OCR semantic assertions passed for 12 states.
- App bundle smoke passed in non-activating ready-file mode.
- Installed app bundle at `~/Applications/Foreman.app`.
- LaunchServices resolves `application "Foreman"` to bundle id
  `dev.foreman.app` via:

```bash
osascript -e 'id of application "Foreman"'
```

Evidence:

- `.ai/validation/macos-overlay/headless-snapshots/attention.png`
- `.ai/validation/macos-overlay/headless-snapshots/compose.png`
- `.ai/validation/macos-overlay/headless-snapshots/help.png`
- `.ai/validation/macos-overlay/app-bundle/summary.md`
- `.ai/validation/macos-overlay/verify-summary.md`

## Manual checks still useful

1. `open -a Foreman` visibly opens the panel.
2. App/menu bar says `Foreman`, not `Foreman Overlay`.
3. Footer Settings, app menu Settings, status menu Settings, and Cmd+, visibly
   open Settings above the overlay.
4. Cmd+T cycles themes once and does not play the invalid-command sound.
5. Help overlay pointer scrolling does not move the background agent list.

These are interaction/desktop checks, so they remain best verified by the user
on the active desktop.
