# Validation — popup-startup-cache

## Focused validation

### Cache service and bootstrap hydration

```bash
cargo test startup_cache -- --nocapture
```

Outcome: pass

Covered:
- startup cache round-trip service tests
- missing-cache handling
- popup bootstrap hydration from a fresh cache

### Cached-startup UI provenance

```bash
cargo test render_surfaces_cached_startup_provenance -- --nocapture
```

Outcome: pass

### Compiled-binary popup cache smoke

```bash
cargo test --test runtime_dashboard interactive_binary_uses_cached_inventory_before_slow_live_refresh -- --nocapture
```

Outcome: pass

Notes:
- this failed once inside the full `mise run verify` lane because the smoke used an absolute time budget
- the test was updated to assert that the cached frame appears well before live refresh replacement instead

### Cache write-rate profiling

```bash
cargo test --test runtime_profiling stable_inventory_writes_startup_cache_once_without_slowdowns -- --ignored --nocapture
```

Outcome: pass

Covered:
- startup cache writes stay bounded during a crowded/stable inventory run
- no slow-operation warning is emitted for cache writes

## Full validation ladder

### Fast gate

```bash
mise run check
```

Outcome: pass

### Heavy validation

```bash
mise run verify
```

Outcome: pass

Included:
- quality gate
- integration and runtime smoke suites
- release gauntlet
- crowded profiling smokes
- Docker build
- UX artifact refresh under `.ai/validation/`

### Native provider readiness

```bash
mise run native-preflight
```

Outcome: pass

### Strict real-provider closeout

```bash
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

Outcome: pass

Covered:
- real Claude dashboard/native completion path
- real Codex native hook path
- real Pi dashboard/native completion path
