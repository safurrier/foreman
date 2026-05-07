---
id: macos-overlay-app-bundle-menu-polish-validation
title: Validation — macOS Overlay App Bundle and Menu Polish
---

# Validation

## 2026-05-04

Commands run:

```bash
mise run verify-macos-overlay-app
```

Result:

- Built `apps/macos-overlay/dist/Foreman Overlay.app`.
- Verified `Info.plist` bundle id, executable, and package type.
- Verified executable exists under `Contents/MacOS/foreman-overlay`.
- Verified bundled Foreman control binary exists under `Contents/Resources/foreman`.
- Launched bundled executable in test mode and wrote panel snapshot.
- Summary written to `.ai/validation/macos-overlay/app-bundle/summary.md`.

Remaining manual check:

```bash
mise run install-macos-overlay-app
open -a "Foreman Overlay"
```

Then confirm Spotlight/Raycast can find **Foreman Overlay** after indexing.
