# Learning Log — popup-startup-cache

## What matched the plan

- A startup-only file cache was enough. There was no need for a daemon or a longer-lived runtime service.
- The right seam was bootstrap hydration plus post-refresh persistence, not a second inventory model.
- Marking cached state in the header and preview kept the UX honest while still making popup startup feel faster.

## What diverged from the plan

- The first write policy was too eager. During staged startup refreshes it wrote the cache several times in quick succession.
- The first compiled-binary smoke used an absolute `<1500ms` budget for the cached frame. That passed in focused runs but failed under the full `mise run verify` load.

## Problems encountered and fixes

### Cache writes churned during startup warm-up

- Symptom: the new profiling smoke showed multiple `startup_cache_write outcome=written` lines while the runtime was still filling in previews.
- Fix: treat the cache as a coarse startup snapshot instead of a constantly refreshed artifact. Writes now skip empty/non-visible states and are rate-limited with a hard minimum interval.
- Final policy:
  - freshness window: `15_000ms`
  - write interval floor: `5_000ms`
  - skip writes when there are no sessions or no visible targets

### Cached-startup smoke was too brittle in the heavy suite

- Symptom: `interactive_binary_uses_cached_inventory_before_slow_live_refresh` failed in `mise run verify` even though the cache path was working.
- Root cause: the test asserted on an absolute elapsed time, which was sensitive to machine load.
- Fix: change the proof to a relative one. The cached frame must appear and remain visible well before the live refresh replaces it.

## Retrospective

- The cache feature is small enough to live comfortably inside the existing runtime model.
- The most important guardrail was the write-frequency perf smoke. Without it, the cache would have silently traded startup latency for steady-state churn.
- The compiled-binary popup smoke is worth keeping even with the profiling tests. It catches UX-level regressions that raw timing logs do not.

## One-shot improvements for next time

- Add a tiny reusable test helper for “render before replacement” timing so popup-startup tests do not invent their own timing proof each time.
- If we add more persisted UI/runtime state, define a small cache manifest format up front rather than one file per feature.

## Skill usefulness

- Most useful: `development-debugging`, `testing-core`, `ratatui-tui`
- Useful but secondary: `writing-core`
