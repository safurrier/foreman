# TODO

## Slice 1 — Global hotkey reliability

- [x] Align Carbon hotkey target/handler with dispatcher target.
- [x] Validate shortcut status remains truthful.
- [x] Install/relaunch and manually ask for Ctrl+F verification.

## Slice 2 — Keyboard mode pass-through

- [x] Make flash/help modes bypass text editing pass-through for advertised keys.
- [x] Preserve native search/compose editing for normal text-editing keys.
- [x] Validate slice.

## Slice 3 — UI gauntlet coverage

- [x] Add scripted flash label selection proof.
- [x] Add scripted `?` help proof.
- [x] Add scripted help scroll proof.
- [x] Validate slice.

## Slice 4 — Terminal activation lifecycle

- [x] Use the persistent terminal activator on focus.
- [x] Refresh activator when terminal activation preferences change.
- [x] Validate slice.

## Slice 5 — Settings hotkey status freshness

- [x] Make hotkey status observable/live in Settings.
- [x] Avoid hotkey reconfiguration on unrelated defaults changes where practical.
- [x] Validate slice.

## Slice 6 — includeAllPanes reload semantics

- [x] Trigger debounced reload when includeAllPanes changes.
- [x] Add test coverage.
- [x] Validate slice.

## Final

- [ ] Run docs/context validation if docs changed.
- [x] Run `mise run validate-macos-overlay-change`.
- [x] Run `mise run install-macos-overlay-app`.
- [x] Mark plan complete and commit.
