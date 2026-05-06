---
id: macos-overlay-unvibecode-architecture-todo
title: TODO — macOS Overlay Un-vibecode Architecture Pass
---

# TODO

## Checkpoint

- [x] Commit current working overlay progress as a safe checkpoint.

## Slice 1 — Settings UI Module

- [x] Move real Settings UI from executable target into `ForemanOverlayUI`.
- [x] Update app settings window to render imported Settings UI.
- [x] Update snapshot target to render the same Settings UI.
- [x] Remove duplicate snapshot-only settings implementation.
- [x] Validate slice.

## Slice 2 — Split executable app shell

- [x] Extract panel lifecycle module.
- [x] Extract app menu/status item module.
- [x] Extract settings window controller.
- [x] Extract keyboard input controller.
- [x] Extract gauntlet hooks.
- [x] Keep behavior unchanged.
- [x] Validate slice.

## Slice 3 — Hotkey diagnostics module

- [x] Create hotkey status/configuration module.
- [x] Centralize persisted vs env override behavior.
- [x] Add diagnostics/logging for registration path.
- [x] Surface current status in Settings or menu-bar label where practical.
- [x] Validate slice.

## Slice 4 — Keyboard event adapter

- [x] Route panel key equivalents through the same command/effect path where possible.
- [x] Keep one mapping from AppKit event to `OverlayKeyboardCommand`.
- [x] Add/adjust tests for key mapping if feasible.
- [x] Validate slice.

## Slice 5 — Terminal activation module

- [x] Extract terminal activation preferences/factory/adapters.
- [x] Add unit tests for preference parsing.
- [x] Keep terminal activation defaults unchanged.
- [x] Validate slice.

## Slice 6 — Snapshot registry

- [x] Make snapshot renderer list states.
- [x] Reduce duplicated state lists in scripts.
- [x] Update docs for snapshot state discovery.
- [x] Validate slice.

## Slice 7 — Split overlay view modules

- [x] Extract agent list/row views.
- [x] Extract detail/PR/preview views.
- [x] Extract footer/compose/help views.
- [x] Remove direct AppKit side effects from UI module if still present.
- [x] Validate slice.

## Slice 8 — Narrow OverlayStore

- [x] Identify store side effects that can move behind a narrower seam.
- [x] Preserve store reducer and published state semantics.
- [x] Add tests for effect decisions.
- [x] Validate slice.

## Final

- [x] Run `mise run verify-macos-overlay`.
- [x] Run `mise run install-macos-overlay-app`.
- [ ] Commit completed architecture pass.
