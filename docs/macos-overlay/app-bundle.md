---
id: macos-overlay-app-bundle
title: macOS Overlay App Bundle
description: Build, install, launch, and smoke-test the native Foreman.app bundle.
index:
  - id: build
  - id: install-for-spotlight-raycast
  - id: development-loop
  - id: app-menu-vs-menu-bar-mode
  - id: validate
---

# Foreman.app

The SwiftPM executable can be wrapped as a local macOS app bundle so Spotlight,
Raycast, Finder, and `open -a` can discover it.

## Build

```bash
mise run build-macos-overlay-app
```

Output:

```text
apps/macos-overlay/dist/Foreman.app
```

The bundle includes:

```text
Foreman.app/
└── Contents/
    ├── Info.plist
    ├── MacOS/foreman-overlay
    └── Resources/foreman
```

The bundled `Resources/foreman` binary is used as the default control API binary
when `FOREMAN_OVERLAY_FOREMAN_PATH` is not set. Development launches can still
point at a custom binary with:

```bash
FOREMAN_OVERLAY_FOREMAN_PATH=$PWD/target/debug/foreman \
apps/macos-overlay/.build/debug/foreman-overlay
```

## Install for Spotlight / Raycast

```bash
mise run install-macos-overlay-app
```

This is the local "make macOS launch the right Foreman" reset command. It:

- builds a fresh app bundle
- stops any running `Foreman.app` or old `Foreman Overlay.app` process
- unregisters known build/prototype/install bundle paths from LaunchServices
- removes stale prototype bundles and the repo-local dist Foreman app
- copies the fresh app to `~/Applications/Foreman.app`
- registers only the installed app and nudges Spotlight/icon caches

The installed app lives at:

```text
~/Applications/Foreman.app
```

Launch with:

```bash
open -a Foreman
```

If Spotlight/Raycast still show stale results, quit/reopen Raycast or wait for
Spotlight indexing. Inspect the installed app with:

```bash
mdls ~/Applications/Foreman.app
```

The install task should leave only the installed app discoverable. A quick check:

```bash
find ~/Applications /Applications apps/macos-overlay/dist -maxdepth 2 -name 'Foreman*.app' -print 2>/dev/null
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -dump \
  | rg 'path:.*Foreman.*\\.app|identifier:\\s+dev\\.foreman\\.app|Foreman Overlay'
```

Expected result: `~/Applications/Foreman.app` is the only Foreman app candidate
and the only `dev.foreman.app` LaunchServices registration.

## Development loop

Use SwiftPM directly for fast compile/test cycles:

```bash
swift test --package-path apps/macos-overlay
```

Use the required overlay validation lane before pushing changes that touch Swift,
the app bundle, keyboard/focus behavior, screenshots, or the Rust control API:

```bash
mise run validate-macos-overlay-change
```

That lane covers Rust control API unit tests, Swift tests, the fake-Foreman UI
gauntlet, real tmux smoke, headless snapshots/OCR, and app-bundle smoke.

After validation, run the install task again before manual Raycast/Spotlight
smoke testing:

```bash
mise run install-macos-overlay-app
open -a Foreman
```

This final install matters because validation builds and runs a repo-local dist
bundle; the install task removes/unregisters that bundle so launchers do not
rediscover a stale development copy.

## App menu vs menu-bar mode

When launched from the `.app`, Foreman defaults to regular app activation
so the top-left macOS menu becomes **Foreman** while active. It also
shows the overlay panel on app launch by default, because Spotlight/Raycast
launches should produce visible feedback.

If the app is already running, launching it again from Spotlight/Raycast or
`open -a Foreman` reopens the overlay panel. Re-activating Foreman from the app
switcher also restores the panel when it is hidden, so Esc/click-away dismissal
is recoverable without the global hotkey.

When launched as a raw SwiftPM executable, it defaults to accessory/menu-bar mode.
Override with:

```bash
FOREMAN_OVERLAY_ACTIVATION_POLICY=regular
FOREMAN_OVERLAY_ACTIVATION_POLICY=accessory
```

Suppress panel display for direct bundled-executable smoke runs with:

```bash
FOREMAN_OVERLAY_SHOW_ON_LAUNCH=0 \
FOREMAN_OVERLAY_RESTORE_ON_ACTIVATE=0 \
FOREMAN_OVERLAY_ACTIVATION_POLICY=accessory \
~/Applications/Foreman.app/Contents/MacOS/foreman-overlay
```

The app menu includes:

- About Foreman
- Settings…
- Hide / Hide Others / Show All
- Quit Foreman

The Foreman menu includes:

- Open Foreman
- Refresh Agents
- Cycle Theme
- Settings…

The menu-bar status item remains available in both modes.

After focusing a tmux pane, Foreman defaults to bringing the terminal app back to
the foreground. Set `FOREMAN_OVERLAY_TERMINAL_ACTIVATION=none` to disable this,
or use `ghostty`, `iterm`, `terminal`, `wezterm`, `alacritty`, `kitty`, or
`bundle:<bundle-id>` to pin a terminal.

## Validate

```bash
mise run verify-macos-overlay-app
```

This checks bundle shape, plist values, bundled executables, and launches the
bundled executable in non-activating ready-file mode. Routine screenshot proof is
handled by the separate headless snapshot renderer so app-bundle verification
does not pop a panel in front of the user's desktop.
