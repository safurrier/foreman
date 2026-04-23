# TODO — startup-loading-flash-hint

## Planning

- [x] Confirm startup is still blocking on initial inventory
- [x] Confirm flash popup is obscuring target rows
- [x] Pick the runtime-first startup approach instead of a separate bootstrap renderer

## Implementation

- [x] Add `startup_loading` state and first-paint support
- [x] Move initial inventory onto the async refresh path
- [x] Add lighter startup preview capture behavior
- [x] Remove the blocking flash popup
- [x] Replace it with inline labels and a smaller non-blocking hint
- [x] Update docs/spec text for startup and flash behavior

## Validation

- [x] Run focused startup and flash tests
- [x] Run `mise run check`
- [x] Run `mise run verify`
- [x] Run `mise run native-preflight`
- [x] Run strict `mise run verify-native`

## Closeout

- [x] Update `LEARNING_LOG.md`
- [x] Update `VALIDATION.md`
- [x] Mark `META.yaml` complete
