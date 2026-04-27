# Implementation Plan

1. Revert visible-target/window-row elision while keeping quiet singleton count rendering.
2. Update state/reducer tests to expect explicit singleton window rows again.
3. Update runtime/profiling navigation and target-count expectations.
4. Update `SPEC.md` so the contract says tree topology is visible, with quiet singleton metadata.
5. Run focused tests, `mise run check`, and `mise run verify-ux`.
6. Push PR update and poll CI/reviews.


## Popup Consistency Extension

1. Make layout selection popup-aware so typical popup dimensions use the medium side-by-side layout.
2. Keep truly tiny terminals compact.
3. Add render and live runtime tests for popup side-by-side layout.
4. Add a popup screenshot to `mise run verify-ux`.
