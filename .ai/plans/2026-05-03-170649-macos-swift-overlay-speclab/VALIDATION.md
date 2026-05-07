---
id: macos-swift-overlay-speclab-validation
title: Validation Log
description: >
  Evidence gathered during the Swift macOS overlay speclab/research pass.
---

# Validation — macos-swift-overlay-speclab

## 2026-05-03 — Research/spec validation

Commands run:

```bash
mise run plan -- macos-swift-overlay-speclab
```

Repo evidence reviewed:

- `docs/workflows.md` — Foreman validation ladder, `.ai` policy, real tmux and
  real harness proof patterns.
- Prior plan `.ai/plans/2026-05-03-140650-macos-popup-raycast-spike/` — control
  API and Raycast spike research, used as prerequisite context.
- Existing architecture docs from previous pass: `SPEC.md`, `docs/architecture.md`,
  `src/cli.rs`, `src/adapters/tmux.rs`, `src/app/state.rs`.

External sources reviewed:

- Apple Spotlight UX:
  - https://support.apple.com/guide/mac-help/search-with-spotlight-mchlp1008/mac
  - https://support.apple.com/en-il/guide/mac-help/mh26783/mac
- Raycast UX/settings:
  - https://manual.raycast.com/search-bar
  - https://manual.raycast.com/settings
- Alfred overview:
  - https://www.alfredapp.com/help/overview/
- Command palette UX pattern:
  - https://uxpatterns.dev/patterns/advanced/command-palette
- Swift global hotkey libraries:
  - https://github.com/sindresorhus/KeyboardShortcuts
  - https://github.com/soffes/HotKey
- Swift/AppKit/SwiftUI Apple docs:
  - https://developer.apple.com/documentation/appkit/nspanel
  - https://developer.apple.com/documentation/SwiftUI/MenuBarExtra
  - https://developer.apple.com/documentation/SwiftUI/Managing-model-data-in-your-app
  - https://developer.apple.com/documentation/appkit/nsevent/addglobalmonitorforevents%28matching%3Ahandler%3A%29
  - https://developer.apple.com/documentation/Foundation/Process
  - https://developer.apple.com/documentation/testing
- Swift testing libraries:
  - https://github.com/pointfreeco/swift-snapshot-testing
  - https://github.com/nalexn/ViewInspector

No code was changed and no Swift project exists yet, so no build/test gate was
run. The validated output is the research/spec/implementation plan artifacts.
