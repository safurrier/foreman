# TODO — popup-startup-cache

## Planning

- [x] Confirm startup cache should stay file-based and startup-only
- [x] Identify current bootstrap and refresh seams
- [x] Choose cache freshness and write-rate limits

## Implementation

- [x] Add startup cache path/runtime config support
- [x] Implement cache load/write service
- [x] Seed runtime bootstrap from fresh cache when available
- [x] Surface cached-startup provenance in the UI
- [x] Rate-limit or skip unnecessary cache writes

## Validation

- [x] Add focused cache-startup tests
- [x] Add focused cache-write profiling test
- [x] Run `mise run check`
- [x] Run `mise run verify`
- [x] Run `mise run native-preflight`
- [x] Run strict `mise run verify-native`

## Closeout

- [x] Update `LEARNING_LOG.md`
- [x] Update `VALIDATION.md`
- [x] Mark `META.yaml` complete
