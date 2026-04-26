# Implementation — UI polish hardening

1. Add UI config explicit-key tracking in config loading and runtime config.
2. Apply persisted theme/sort only when the corresponding config key is omitted.
3. Replace immediate UI preference writes with dirty/debounced runtime flushing and shutdown flush.
4. Make preference temp writes unique and add reset/load doctor helpers.
5. Extend startup cache stale result with inventory counts for doctor.
6. Add PR refresh command, state, reducer action, runtime lookup forcing, and render feedback.
7. Enrich notification requests with session/window labels.
8. Update docs/plan artifacts and run focused + full validation.
