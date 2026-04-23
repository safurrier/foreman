# Validation — release-hygiene-final-pass

## Planned validation

### Focused

- actionable-pane reducer/state regressions
- preview provenance adapter/runtime/render regressions
- workflow/CI evidence-path checks

### Full

```bash
mise run check
mise run verify
mise run native-preflight
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

## Status

Complete.

## Executed validation

### Cleanup / focused checks

```bash
rg -n -i "tmuxcc" .
python3 -m py_compile .mise/tasks/verify-ux .mise/tasks/verify-release scripts/lib.py
cargo test --lib -- --nocapture
cargo test --test notification_runtime runtime_uses_configured_notification_backend_order -- --nocapture
cargo test --test release_gauntlet release_integration_gauntlet_proves_pr_notifications_and_graceful_degradation -- --nocapture
mise run verify-ux --capture-only
```

Results:

- `rg -n -i "tmuxcc" .` returned no matches.
- Workflow scripts compiled cleanly.
- Library tests passed.
- The focused notification runtime regression passed after moving the proof to a non-selected actionable pane.
- The focused release gauntlet regression passed after moving selection to `betawork` and waiting on refresh cycles rather than selected-pane preview text.
- `mise run verify-ux --capture-only` passed after fixing the generated VHS tape syntax.

### Full closeout

```bash
mise run check
mise run verify
mise run native-preflight
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

Results:

- `mise run check`: passed
- `mise run verify`: passed
- `mise run native-preflight`: passed
- strict `mise run verify-native`: passed

### Notable evidence

- Stable validation outputs now land under:
  - `.ai/validation/ux/`
  - `.ai/validation/release/`
- CI and release uploads now fail loudly if those expected artifacts are missing.
- Strict real native E2E passed for:
  - Claude
  - Codex
  - Pi
