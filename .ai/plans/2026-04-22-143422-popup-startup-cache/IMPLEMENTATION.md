---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
---

# Implementation — popup-startup-cache

## Approach

- Add a small startup-cache service that reads and writes a JSON inventory snapshot under Foreman state.
- Seed `prepare_runtime_bootstrap()` from a fresh cache entry, but keep `startup_loading=true` until live tmux refresh lands.
- Track cache provenance in `AppState` so header and preview can say the first frame came from cache.
- Write cache snapshots only after successful live refreshes, and rate-limit/skip unchanged writes so the write path stays cheap.
- Add one cache-first startup smoke and one ignored profiling smoke for cache write rate and timing.

## Steps

1. Add runtime/config path support for a startup cache location and policy.
2. Implement a startup cache service with fresh-load, atomic write, and tmux-socket keying.
3. Hydrate runtime bootstrap from cache and clear cache provenance once live inventory arrives.
4. Add header/details copy for cached startup state.
5. Add cache write logging, skip logic, and perf assertions.
6. Run focused tests, then `mise run check`, `mise run verify`, `mise run native-preflight`, and strict `mise run verify-native`.
