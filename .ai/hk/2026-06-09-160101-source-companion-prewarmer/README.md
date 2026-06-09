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
- None recorded.

## Plan
- Implement the first ADR 0004 slices: source snapshot store, fixture-backed freshness/diagnostic tests, explicit companion/prewarmer commands, runtime read-only snapshot loading behind opt-in config, and real Mac↔Coder validation using the available Coder SSH connection. Preserve one-shot SSH as fallback and keep popup key-latency budgets covered.

## Decisions and spec reflection
- Implemented the first ADR 0004 slice as a read-only snapshot source, not a daemon or reverse tunnel: SourceSnapshotStore owns atomic JSON snapshot lifecycle; sources snapshot writes a source-local AgentsResponse envelope; snapshot source config lets another Foreman instance render cached rows; live actions still require SSH/future companion transport.
  - Spec: updated: Spec/docs updated or verified.; refs: docs/operator-guide.md, docs/workflows.md

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

## Readiness
- context: info — no context recorded; okay for trivial work, add hk context if it prevents rediscovery
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: pass — validation evidence with rationale recorded
- review: pass — external-enough review recorded
- profile-check:fast-gate: pass — required profile check recorded: fast-gate (matched docs/operator-guide.md, docs/workflows.md, src/cli.rs, +5 more)
- profile-review:codex-review: pass — required profile review recorded: codex-review (matched src/cli.rs, src/lib.rs, src/runtime.rs, +3 more)

## Review
- subagent / reviewer: Reviewed source snapshot/prewarmer implementation. Fixed blockers: sources doctor now preserves successful snapshot warm/stale diagnostics; runtime aggregation carries successful snapshot diagnostics into operator UI diagnostics; source summary staleness reflects snapshot warm/stale diagnostics; live Coder smoke now asserts both local and Mac rows. paths: src/source_snapshots.rs, src/sources.rs, src/cli.rs, +4 more. [accepted]
- subagent / reviewer: Reviewed regenerated UX artifacts and live Coder snapshot smoke artifact as validation outputs from passing verify-ux and scripts/smoke-source-snapshot-coder.sh. paths: .ai/validation/ux/foreman-ux-diagnostic.gif, .ai/validation/ux/foreman-ux-flash.png, .ai/validation/ux/foreman-ux-gruvbox.png, +5 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Codex-review equivalent coverage for snapshot source implementation: checked source snapshot store, source config/provider integration, CLI commands, runtime diagnostics propagation, and live smoke script. Review blockers were fixed before this record. paths: src/source_snapshots.rs, src/sources.rs, src/cli.rs, +3 more. [accepted]
