# HK export: `2026-06-13-133659-harness-status-explain-pi-subagents`

This directory is a generated review/handoff package from the Harness Kit ledger. Do not hand-edit it; update HK with `hk plan`, `hk decide`, `hk validate`, `hk review add`, and `hk sync`, then regenerate.

## Freshness
Validate this export against local HK state with:

```bash
hk export --format handoff-dir --output .ai/hk/2026-06-13-133659-harness-status-explain-pi-subagents --target . --check
```

Historical hand-authored slice plans live under `.ai/plans/`; new Harness Toolkit repo work should use HK and generated `.ai/hk/` exports.

## Handoff

## Summary
- Work: `2026-06-13-133659-harness-status-explain-pi-subagents`
- Branch: `feat/harness-status-explain-pi-subagents`

## Context
- Research-plan context: Herdr keeps one status authority per pane; Pi lifecycle hooks can be authoritative, while Claude/Codex hooks are session/provenance-only in Herdr because they miss transitions. Foreman currently has native-over-compat precedence by spec, a Pi active-accounted event reducer when run/process ids are present, external extension-card providers keyed by workspace/repo, and hard-coded compatibility StatusHints for Claude/Codex/Pi. Plan should preserve current native precedence unless an explicit spec decision changes it.

## Plan
- Implement four Foreman harness-status improvements from the Herdr/pi-subagents research spike: add diagnostics that explain Pi active-accounted vs legacy hook mode; add a read-only pi-subagents extension-card/provider that reads structured async run status instead of terminal text; add manifest-style explainable compatibility rules for Claude/Codex/Pi fallback; and add idle smoothing/retry-hold/release-oriented tests around compatibility and Pi lifecycle transitions. Keep the slice in a scoped worktree, preserve native hook precedence unless explicitly changed by documented policy, and validate with focused unit/contract tests, setup/doctor checks, mise run check, and native-preflight/verify-native where real harness behavior is touched.
- Expand PR #29: move pi-subagents structured status parsing into an integration-owned typed activity seam, merge fresh workspace-matched subagent activity into Pi native overlay so active child runs keep/promote the parent Pi pane Working without letting extension cards become a status authority. Preserve native Error/NeedsAttention precedence, respect IntegrationPreference::Compatibility, keep stale/wrong-workspace evidence from promoting status, and have the read-only card render from the same typed activity source.

## Decisions and spec reflection
- Pi subagent cards are read-only explanatory pane cards; native overlay remains the status authority and this PR does not externalize compatibility rules as user config.
- Pi subagent activity is native Pi evidence, not a generic extension-card status authority. Merge policy: fresh active subagent runs may promote Pi Idle/Unknown/compatibility to Working, but native Error/NeedsAttention wins; stale or wrong-workspace subagent files do not affect parent pane status.
  - Spec: not-needed: Spec/docs update not needed.

## Learning
- Architecture polish review found and the branch fixed a native summary accounting bug: when Pi subagent activity promotes a compatibility Pi pane to native status, fallback_to_compatibility must be decremented so runtime/doctor summaries do not report the same pane as both native and fallback. The review also required explicit policy/tests/docs for subagent needs_attention promotion as Pi-native evidence.

## Gaps
- None recorded.

## Validation evidence
- `git status --short --branch`: pass (exit 0) — validates: Planning checkpoint proves the implementation branch/worktree is clean and scoped before coding. — `<local HK state not exported>`
- `cargo test -q compatibility_status doctor extension_cards pi_subagents`: fail (exit 1) — attempted to validate: Compatibility explanation and doctor/card diagnostics tests pass. — `<local HK state not exported>`
- `cargo test -q native_events pi_hook`: fail (exit 1) — attempted to validate: Focused native lifecycle tests pass for active-run accounting and Pi hook behavior. — `<local HK state not exported>`
- `mise run native-preflight`: pass (exit 0) — validates: Native setup preflight still passes after native diagnostic changes. — `<local HK state not exported>`
- `cargo test -q pi_hook`: pass (exit 0) — validates: Pi hook bridge tests pass for legacy and active-accounted signal materialization. — `<local HK state not exported>`
- `cargo test -q native_events`: pass (exit 0) — validates: Native event reducer tests pass for active-run accounting. — `<local HK state not exported>`
- `cargo test -q extension_cards`: pass (exit 0) — validates: Extension and pi-subagents card tests pass. — `<local HK state not exported>`
- `cargo test -q doctor`: pass (exit 0) — validates: Doctor diagnostics tests pass, including compatibility explanations and Pi native signal provenance. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate passes after harness diagnostics and pi-subagents card changes. — `<local HK state not exported>`
- `mise run native-preflight`: pass (exit 0) — validates: Native preflight passes; hook argv/native lifecycle behavior was not changed, but integration files were touched. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate passes after harness diagnostics and pi-subagents card changes. — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: verify harness diagnostics and pane-scoped pi-subagents extension-card cache after review fix — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: verify harness diagnostics and pane-scoped pi-subagents extension-card cache after review fix — `<local HK state not exported>`
- `cargo test --test release_gauntlet release_integration_gauntlet_proves_pr_notifications_and_graceful_degradation -- --exact`: pass (exit 0) — validates: rerun release gauntlet test that failed transiently during commit hook; proves PR panel path still passes — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: verify Pi subagent activity native-status augmentation and shared typed activity card source — `<local HK state not exported>`
- `mise run native-preflight`: pass (exit 0) — validates: native harness smoke after adding Pi subagent activity as native status evidence — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: verify Pi subagent native-status activity merge after architecture review fixes — `<local HK state not exported>`
- `mise run native-preflight`: pass (exit 0) — validates: native harness smoke after Pi subagent native-status merge and review fixes — `<local HK state not exported>`

## Readiness
- context: info — context recorded
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: pass — validation evidence with rationale recorded
- review: pass — external-enough review recorded

## Review
- subagent / architecture-polish-review-subagent [architecture-polish-review]: B- plan review accepted; plan tightened around pane-scoped pi-subagents cards, read-only explanatory status, internal compatibility rule module, concrete Pi signal provenance diagnostics, and validation gating. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Accepted after follow-up. Prior blocker was same-workspace Pi/non-Pi extension-card cache leakage; current diff keys selected extension cache and refreshing state by workspace plus actionable pane key, adds regression coverage, and preserves native/compatibility precedence. Reviewer ran git diff --check and focused cargo tests for pane scoping, pi_subagents, compatibility explanation, and Pi provenance. paths: src/app/action.rs, src/app/mod.rs, src/app/reducer.rs, +13 more. [accepted]
- subagent / reviewer-fresh-context: Accepted follow-up review. Documentation change is consistent with implemented Pi provenance labels and read-only pi-subagents card behavior; no blocking findings. paths: docs/operator-guide.md. [accepted]
- subagent / reviewer-fresh-context: Architecture polish review completed by subagent 019ec31c-ef8a-7b82-a6b9-43791e2d033c. Initial grade B+ and not quite merge-ready due stale compatibility fallback accounting for subagent-promoted panes plus implicit subagent attention policy. Fixed fallback accounting, updated tests to require fallback_to_compatibility=0 after promotion, added failed-only and attention-promotion regression tests, and documented subagent attention as Pi-native evidence. Remaining watch item is possible future per-refresh caching if async run counts grow. paths: SPEC.md, docs/operator-guide.md, src/integrations/mod.rs, +4 more. [accepted]
