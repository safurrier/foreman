# TODO — release-hygiene-final-pass

## Actionable-pane semantics

- [x] Split aggregate row workspace info from actionable pane workspace context
- [x] Make notification suppression use resolved actionable pane identity
- [x] Make PR lookup and diagnostics use actionable pane workspace context
- [x] Add direct regression tests for session and window selection behavior

## Release hygiene

- [x] Move validation evidence to a stable output root
- [x] Update CI and release artifact upload paths
- [x] Fail loudly when expected evidence is missing
- [x] Update README and workflow docs for the new evidence root

## Degraded preview observability

- [x] Add per-pane preview provenance to the pane model
- [x] Log capture failures with pane ids
- [x] Surface degraded preview provenance in the preview panel
- [x] Keep capture failure behavior soft-fail

## Cleanup

- [x] Remove stray benchmark-name references from the repo

## Validation

- [x] Focused state/reducer/runtime/render/workflow tests
- [x] `mise run check`
- [x] `mise run verify`
- [x] `mise run native-preflight`
- [x] Strict `mise run verify-native`

## Closeout

- [x] Update `LEARNING_LOG.md`
- [x] Update `VALIDATION.md`
- [x] Mark `META.yaml` complete
