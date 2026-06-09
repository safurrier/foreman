# HK export: `2026-06-09-145453-source-companion-relay-design`

This directory is a generated review/handoff package from the Harness Kit ledger. Do not hand-edit it; update HK with `hk plan`, `hk decide`, `hk validate`, `hk review add`, and `hk sync`, then regenerate.

## Freshness
Validate this export against local HK state with:

```bash
hk export --format handoff-dir --output .ai/hk/2026-06-09-145453-source-companion-relay-design --target . --check
```

Historical hand-authored slice plans live under `.ai/plans/`; new Harness Toolkit repo work should use HK and generated `.ai/hk/` exports.

## Handoff

## Summary
- Work: `2026-06-09-145453-source-companion-relay-design`
- Branch: `design/source-companion-relay`

## Context
- None recorded.

## Plan
- Design the post-#25 Foreman source companion / relay architecture for bidirectional Mac ↔ Coder visibility and prewarmed source state. Produce an ADR comparing reverse SSH tunnel, source companion, relay/cache service, and prewarmer-only paths; include sequence diagrams, failure modes, cache freshness/validation plan, and update docs/AGENTS index as needed.

## Decisions and spec reflection
- Documented ADR 0004 as the proposed next architecture: keep one-shot SSH as fallback, design/prototype source companion snapshots first, and treat reverse tunnels/relays as transports for the companion protocol rather than separate product models.
  - Spec: not-needed: Spec/docs update not needed.; refs: docs/decisions/0004-source-companion-relay.md

## Learning
- None recorded.

## Gaps
- None recorded.

## Validation evidence
- `uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py .`: pass (exit 0) — validates: Docs-only source companion/relay design validates links, frontmatter, and docs index after adding ADR 0004. — `<local HK state not exported>`
- `uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py .`: pass (exit 0) — validates: Docs-only source companion/relay ADR update: verify docs links/frontmatter/index instead of full Rust gate. — `<local HK state not exported>`
- `uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py .`: pass (exit 0) — validates: Polish ADR 0004 after architecture review: verify docs links/frontmatter/index. — `<local HK state not exported>`

## Readiness
- context: info — no context recorded; okay for trivial work, add hk context if it prevents rediscovery
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: fail — validation evidence is stale for current changed paths; rerun hk validate or dangerously-skip validation. Current changed paths: .pi/state/codex-pr-review-monitor.json.
- review: fail — accepted review does not cover current changed paths; run a targeted follow-up review with `hk review add --path PATH ...` or dangerously-skip review. Current changed paths: .pi/state/codex-pr-review-monitor.json.
- profile-check:fast-gate: pass — required profile check recorded: fast-gate (matched docs/AGENTS.md, docs/architecture.md, docs/decisions/0004-source-companion-relay.md)

## Review
- subagent / reviewer: Reviewed ADR 0004 source companion/relay design. Addressed findings: clarified companion snapshot entries are source-local and wrapped by the consumer, split invalid cargo validation command into valid filters, and removed overcommitted signed-snapshot wording. paths: docs/decisions/0004-source-companion-relay.md, docs/AGENTS.md, docs/architecture.md. [accepted]
- subagent / architecture-polish-review: Architecture polish review graded ADR 0004 B+ and identified improvements. Applied them: added SnapshotSnapshotStore ownership/lifecycle invariants, protocol identity/action trust rules, concrete popup/source validation acceptance criteria, and a smaller seam-by-seam implementation ladder. paths: docs/decisions/0004-source-companion-relay.md. [accepted]
