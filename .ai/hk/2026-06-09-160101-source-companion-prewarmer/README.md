# HK export: `2026-06-09-160101-source-companion-prewarmer`

This directory is a generated review/handoff package from the Harness Kit ledger. Do not hand-edit it; update HK with `hk plan`, `hk decide`, `hk validate`, `hk review add`, and `hk sync`, then regenerate.

## Freshness
Validate this export against local HK state with:

```bash
hk export --format handoff-dir --output .ai/hk/2026-06-09-160101-source-companion-prewarmer --target . --check
```

Historical hand-authored slice plans live under `.ai/plans/`; new Harness Toolkit repo work should use HK and generated `.ai/hk/` exports.

## Handoff

## Summary
- Work: `2026-06-09-160101-source-companion-prewarmer`
- Branch: `feature/source-companion-prewarmer`

## Context
- User explicitly wants Phase 6 as the definition of done for PR #27 and does not want six separate PRs. Keep all follow-up implementation on branch feature/source-companion-prewarmer / PR #27; do not merge it yet. Prefer Python live integration harness over additional complex shell scripts.

## Plan
- Implement the first ADR 0004 slices: source snapshot store, fixture-backed freshness/diagnostic tests, explicit companion/prewarmer commands, runtime read-only snapshot loading behind opt-in config, and real Mac↔Coder validation using the available Coder SSH connection. Preserve one-shot SSH as fallback and keep popup key-latency budgets covered.

## Decisions and spec reflection
- Implemented the first ADR 0004 slice as a read-only snapshot source, not a daemon or reverse tunnel: SourceSnapshotStore owns atomic JSON snapshot lifecycle; sources snapshot writes a source-local AgentsResponse envelope; snapshot source config lets another Foreman instance render cached rows; live actions still require SSH/future companion transport.
- Expanded PR #27 from Phase 1 snapshot foundation to the full Phase 2-6 source companion definition of done in one PR: prewarmer loop, companion registration, reverse/live transport, live focus/send action transport, and display activation parity. Validation smokes should move from shell glue toward a Python integration harness.
- PR #27 now implements the source companion architecture through the code seams for Phase 2-6: prewarmer loop, registration metadata, companion live transport, gated focus/send actions, and activation-command configuration. Live Coder reverse-actions validation is not complete because the current Coder SSH proxy accepts ssh -R but does not expose a reachable forwarded port; this is documented as a transport/environment blocker rather than a protocol blocker.
  - Spec: updated: Spec/docs updated or verified.; refs: docs/operator-guide.md, docs/workflows.md
  - Spec: updated: Spec/docs updated or verified.; refs: docs/decisions/0004-source-companion-relay.md
  - Spec: updated: Spec/docs updated or verified.; refs: docs/decisions/0004-source-companion-relay.md, docs/operator-guide.md

## Learning
- None recorded.

## Gaps
- None recorded.

## Validation evidence
- `bash -lc 'uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py . && cargo test --lib --quiet && cargo test --test runtime_profiling -- --ignored && scripts/smoke-popup-key-latency.sh'`: pass (exit 0) — validates: Implement source snapshot store and read-only snapshot source with CLI commands; validate unit coverage, runtime profiling, popup key latency, and docs verification. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Full fast gate after source snapshot store/source config/CLI/runtime/docs changes. — `<local HK state not exported>`
- `cargo install --path . --locked --force`: pass (exit 0) — validates: Installed updated Foreman locally after snapshot source implementation. — `<local HK state not exported>`
- `scripts/smoke-source-snapshot-coder.sh`: pass (exit 0) — validates: Live Coder integration: Mac writes snapshot, copies to Coder, Coder Foreman renders local Coder rows plus Mac snapshot rows. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after review fixes: sources doctor/runtime now preserve successful snapshot diagnostics and Coder smoke enforces both local and Mac rows. — `<local HK state not exported>`
- `scripts/smoke-source-snapshot-coder.sh`: pass (exit 0) — validates: Live Coder snapshot smoke after review fixes: enforces both local Coder rows and Mac snapshot rows. — `<local HK state not exported>`
- `uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py .`: pass (exit 0) — validates: Docs verifier after expanding PR #27 with companion transport/prewarmer docs. — `<local HK state not exported>`
- `cargo test --lib source --quiet`: pass (exit 0) — validates: Focused source companion implementation tests: snapshot store registration, companion source/client, source config, and CLI companion add coverage. — `<local HK state not exported>`
- `scripts/source_companion_live_smoke.py companion-local --artifact-dir .ai/validation/source-companion/companion-local --json --no-build`: pass (exit 0) — validates: Python live harness local companion smoke: live JSON-line companion source returns source-scoped rows without SSH. — `<local HK state not exported>`
- `scripts/source_companion_live_smoke.py coder-snapshot --artifact-dir .ai/validation/source-companion/coder-snapshot --json --no-build`: pass (exit 0) — validates: Live Coder snapshot smoke through Python harness: Coder renders local rows plus Mac cached snapshot rows using installed Coder Foreman. — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Fast gate after expanding PR #27 to source companion protocol, registration, prewarmer, and Python validation harness. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after fixing companion clippy large-error warnings. — `<local HK state not exported>`
- `uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py .`: pass (exit 0) — validates: Docs verifier after ADR 0004 implementation status update. — `<local HK state not exported>`
- `cargo test --lib source --quiet`: pass (exit 0) — validates: Focused source companion tests after review fixes: all-panes propagation and companion socket timeouts. — `<local HK state not exported>`
- `scripts/source_companion_live_smoke.py companion-local --artifact-dir .ai/validation/source-companion/companion-local --json --no-build`: pass (exit 0) — validates: Python companion-local smoke after all-panes propagation fix: local and companion rows both include all panes. — `<local HK state not exported>`
- `scripts/source_companion_live_smoke.py coder-snapshot --artifact-dir .ai/validation/source-companion/coder-snapshot --install-coder --json`: pass (exit 0) — validates: Live Coder snapshot smoke after review fixes and candidate reinstall. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after reviewer blocker fixes for companion all-panes propagation and socket read/write timeouts. — `<local HK state not exported>`

## Readiness
- context: info — context recorded
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: fail — validation evidence is stale for current changed paths; rerun hk validate or dangerously-skip validation. Current changed paths: .pi/state/codex-pr-review-monitor.json.
- review: fail — accepted review does not cover current changed paths; run a targeted follow-up review with `hk review add --path PATH ...` or dangerously-skip review. Current changed paths: .pi/state/codex-pr-review-monitor.json.
- profile-check:fast-gate: pass — required profile check recorded: fast-gate (matched docs/decisions/0004-source-companion-relay.md, docs/operator-guide.md, docs/workflows.md, +8 more)
- profile-review:codex-review: pass — required profile review recorded: codex-review (matched scripts/smoke-source-snapshot-coder.sh, src/cli.rs, src/lib.rs, +5 more)

## Review
- subagent / reviewer: Reviewed source snapshot/prewarmer implementation. Fixed blockers: sources doctor now preserves successful snapshot warm/stale diagnostics; runtime aggregation carries successful snapshot diagnostics into operator UI diagnostics; source summary staleness reflects snapshot warm/stale diagnostics; live Coder smoke now asserts both local and Mac rows. paths: src/source_snapshots.rs, src/sources.rs, src/cli.rs, +4 more. [accepted]
- subagent / reviewer: Reviewed regenerated UX artifacts and live Coder snapshot smoke artifact as validation outputs from passing verify-ux and scripts/smoke-source-snapshot-coder.sh. paths: .ai/validation/ux/foreman-ux-diagnostic.gif, .ai/validation/ux/foreman-ux-flash.png, .ai/validation/ux/foreman-ux-gruvbox.png, +5 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Codex-review equivalent coverage for snapshot source implementation: checked source snapshot store, source config/provider integration, CLI commands, runtime diagnostics propagation, and live smoke script. Review blockers were fixed before this record. paths: src/source_snapshots.rs, src/sources.rs, src/cli.rs, +3 more. [accepted]
- subagent / reviewer [codex-review]: Reviewed expanded PR #27 source companion implementation. Blockers found: companion all-panes propagation and blocking companion socket reads. Both were fixed: CompanionSource now carries include_non_agents/all_panes into the protocol request, companion server sets per-connection read/write timeouts. Follow-ups noted: trust UX for allow_send/token, Coder proxy reverse-forward limitation, and fallback error detail. paths: src/source_companion.rs, src/source_snapshots.rs, src/sources.rs, +6 more. [accepted]
- subagent / reviewer: Targeted review plus fixes for source companion Phase 2-6 work. Review-only subagent found two blockers; implementation now addresses both and validation passed afterwards. paths: src/source_companion.rs, src/source_snapshots.rs, src/sources.rs, +7 more. [accepted]
- subagent / reviewer [codex-review]: Reviewed module export change for source_companion; src/lib.rs only exposes the new source_companion module used by cli/sources tests. paths: src/lib.rs. [accepted]
- subagent / reviewer: Reviewed src/lib.rs module export for source companion. paths: src/lib.rs. [accepted]
