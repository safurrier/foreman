# HK export: `2026-07-14-072828-ghostty-display-identity`

This directory is a generated review/handoff package from the Harness Kit ledger. Do not hand-edit it; update HK with `hk plan`, `hk decide`, `hk validate`, `hk review add`, and `hk sync`, then regenerate.

## Freshness
Validate this export against local HK state with:

```bash
hk export --format handoff-dir --output .ai/hk/2026-07-14-072828-ghostty-display-identity --target . --check
```

Historical hand-authored slice plans live under `.ai/plans/`; new Harness Toolkit repo work should use HK and generated `.ai/hk/` exports.

## Handoff

## Summary
- Work: `2026-07-14-072828-ghostty-display-identity`
- Branch: `feature/ghostty-display-identity`

## Context
- Factory issue 34. Session 3f4b0ee5-6d0a-5845-821b-eed526753f01; resume: pi --session 3f4b0ee5-6d0a-5845-821b-eed526753f01. Implementation commits 8f80876 and b559a29 in this isolated worktree.
- # Issue 34: Registered Ghostty display identity

## Solution shape

Add a source-owned local display-registration module, stored in Foreman's local state area and keyed by source ID. A registration contains a typed provider (`ghostty`), exact terminal UUID, optional tab/window IDs and title for diagnostics only, capture/update timestamps, and an opaque ownership handle. Atomic replacement makes the new handle current; unregister is compare-and-delete so stale cleanup cannot remove a replacement.

Expose `foreman sources display register <source-id> --provider ghostty`, `list`, `doctor`, and `unregister --handle ...` (exact CLI spelling may follow nearby clap conventions). Ghostty capture and focus live behind a provider/activator boundary and call the official AppleScript API. Exact terminal ID is authoritative; title, tty, and pid never select a target. Unsupported platforms/providers and missing or closed terminals return typed diagnostics.

Runtime source focus remains two-stage: tmux focus first, then local display activation. Registered identity is attempted before the existing command. If it is absent or unavailable, run the configured activation command as compatibility fallback. Activation failure remains a warning/diagnostic and never rewrites successful tmux focus as `focus failed`. Companion-host focus uses the same local registry in the companion Foreman process, then its existing `--activation-command` fallback.

Keep source-companion registration JSON backward compatible by adding only serde-defaulted optional identity data or, preferably, leave companion transport registration separate from the new local display registry because display identity must not cross SSH.

## Implementation / TDD plan

1. Characterize current caller and companion focus/activation outcomes and existing registration serialization.
2. Add behavior tests first for typed serialization/backward compatibility, replacement ownership, old-handle unregister rejection, exact-ID AppleScript generation/selection, stale terminal and provider/platform errors, registered-first command fallback, and separate tmux/display outcomes.
3. Implement the local registry/store and explicit display provider boundary with atomic persistence.
4. Add discoverable CLI capture/register/list/doctor/unregister JSON contracts and include registration health in source list/doctor output.
5. Wire caller runtime and companion-host focus through one activation resolver, preserving existing command fallback and structured warnings.
6. Add an isolated macOS Ghostty smoke for capture → title rename → exact focus, with no default tmux-server mutation; document TCC, lifecycle, fallback, health, and ADR 0004 Slice 6 completion.
7. Run focused source snapshot/companion/runtime tests, the new smoke when a suitable live Ghostty terminal is safely available, `mise run check`, `mise run validate-macos-overlay-change`, and `mise run verify`. Record environment blockers instead of disturbing the operator.
8. Run spec-sync, architecture polish, anti-pattern review, and the profile Codex review. Fix concrete P0-P2 findings and repeat only invalidated checks/review once.

## Acceptance rubric

- MUST: Stable exact Ghostty terminal UUID remains the activation target after title changes and cannot confuse duplicate titles. Proof: unit tests plus live macOS smoke when safe.
- MUST: One current registration per source; replacement invalidates old ownership handles; old-handle cleanup cannot delete replacement. Proof: store tests.
- MUST: Closed/missing terminals and unsupported provider/platform emit actionable `source.display.unavailable`-style diagnostics without changing successful tmux focus into focus failure. Proof: resolver/runtime/companion tests.
- MUST: Registered activation precedes existing caller and companion command fallbacks; configs containing only activation commands retain behavior. Proof: fallback tests.
- MUST: Existing registration JSON without display identity continues to deserialize or a tested migration is explicitly documented. Proof: fixture/serde test.
- MUST: CLI can capture/register, list/inspect health in JSON, and unregister only with matching handle. Proof: CLI tests and help inspection.
- MUST: No production target lookup uses title substring, terminal tty, or terminal pid. Proof: diff inspection and provider tests.
- MUST: Required focused commands and broad gates pass or have explicit environment evidence; HK sync/ready/handoff complete before PR.
- QUALITY: `alex-ai-testing-core` verifies behavior-first tests at the correct seams.
- QUALITY: `alex-ai-architecture-polish-review` verifies one clear registry owner, provider boundary, lifecycle, and deletion/test seams.
- QUALITY: `alex-ai-avoiding-ai-antipatterns` checks fallback observability and unnecessary abstractions/shims.
- QUALITY: `spec-sync` verifies ADR/SPEC/interface alignment.
- QUALITY: `alex-ai-writing-core` checks operator and ADR prose.
- QUALITY: `alex-ai-bash-core` checks the macOS smoke script if added.

Verdict: every MUST and selected QUALITY criterion passes; no averaging. A live smoke may be environment-blocked only with a recorded safe reason and deterministic provider/CLI proof; no operator terminal or default tmux server may be disturbed.

## Skill selection ledger

- APPLY:WORKFLOW `alex-ai-hk-plan` — large cross-module runtime/CLI/persistence change requires full HK lifecycle.
- APPLY:WORKFLOW `alex-ai-research` — trace existing caller, companion, registration, and ADR paths before implementation.
- APPLY:QUALITY `alex-ai-testing-core` — task explicitly requires seam-level unit and live-smoke proof.
- APPLY:QUALITY `alex-ai-architecture-polish-review` — new state ownership and provider boundary need architecture review.
- APPLY:QUALITY `alex-ai-avoiding-ai-antipatterns` — fallback and compatibility logic are prone to silent or duplicated behavior.
- APPLY:QUALITY `spec-sync` — public CLI and ADR 0004 contract change.
- APPLY:QUALITY `alex-ai-writing-core` — operator and ADR docs must be direct and accurate.
- APPLY:QUALITY `alex-ai-bash-core` — likely new macOS smoke script.
- SKIP `foreman-swift-overlay-ux` — no Swift overlay UI is in scope.
- SKIP `foreman-swift-overlay-validation` — requested overlay lane is validation evidence, not changed Swift UI harness ownership.
- SKIP `macos-app-design` / `macos-design-guidelines` — this is a command-line AppleScript adapter, not Mac UI design.
- SKIP `alex-ai-ratatui-tui` — focus behavior changes behind the TUI; rendering/input design does not change.
- SKIP `alex-ai-scope-based-hk-validation` — `hk-plan` already owns sizing and lifecycle; avoid duplicate workflow policy.

## Worker launch contract

Provider/model: `openai-codex/gpt-5.6-sol:high`
Reason: hard cross-module Rust persistence, CLI, runtime, companion, macOS adapter, docs, and smoke work.
Availability: inherited worker mapping is `openai-codex/gpt-5.6-sol`; exact model row and live sentinel probe passed on 2026-07-14.
Budget: maxRuntimeMs 7,200,000; 96 turns + 16 grace; tools 240 soft / 320 hard with all tools blocked after hard; control enabled, needs-attention after 300,000ms.
Escalation: stop for architecture/scope/safety decisions not resolved here; one fresh exact-model follow-up at most for bounded fixes; never resume/revive a model-pinned worker.

## Plan
- # Issue 34: Registered Ghostty display identity

## Solution shape

Add a source-owned local display-registration module, stored in Foreman's local state area and keyed by source ID. A registration contains a typed provider (`ghostty`), exact terminal UUID, optional tab/window IDs and title for diagnostics only, capture/update timestamps, and an opaque ownership handle. Atomic replacement makes the new handle current; unregister is compare-and-delete so stale cleanup cannot remove a replacement.

Expose `foreman sources display register <source-id> --provider ghostty`, `list`, `doctor`, and `unregister --handle ...` (exact CLI spelling may follow nearby clap conventions). Ghostty capture and focus live behind a provider/activator boundary and call the official AppleScript API. Exact terminal ID is authoritative; title, tty, and pid never select a target. Unsupported platforms/providers and missing or closed terminals return typed diagnostics.

Runtime source focus remains two-stage: tmux focus first, then local display activation. Registered identity is attempted before the existing command. If it is absent or unavailable, run the configured activation command as compatibility fallback. Activation failure remains a warning/diagnostic and never rewrites successful tmux focus as `focus failed`. Companion-host focus uses the same local registry in the companion Foreman process, then its existing `--activation-command` fallback.

Keep source-companion registration JSON backward compatible by adding only serde-defaulted optional identity data or, preferably, leave companion transport registration separate from the new local display registry because display identity must not cross SSH.

## Implementation / TDD plan

1. Characterize current caller and companion focus/activation outcomes and existing registration serialization.
2. Add behavior tests first for typed serialization/backward compatibility, replacement ownership, old-handle unregister rejection, exact-ID AppleScript generation/selection, stale terminal and provider/platform errors, registered-first command fallback, and separate tmux/display outcomes.
3. Implement the local registry/store and explicit display provider boundary with atomic persistence.
4. Add discoverable CLI capture/register/list/doctor/unregister JSON contracts and include registration health in source list/doctor output.
5. Wire caller runtime and companion-host focus through one activation resolver, preserving existing command fallback and structured warnings.
6. Add an isolated macOS Ghostty smoke for capture → title rename → exact focus, with no default tmux-server mutation; document TCC, lifecycle, fallback, health, and ADR 0004 Slice 6 completion.
7. Run focused source snapshot/companion/runtime tests, the new smoke when a suitable live Ghostty terminal is safely available, `mise run check`, `mise run validate-macos-overlay-change`, and `mise run verify`. Record environment blockers instead of disturbing the operator.
8. Run spec-sync, architecture polish, anti-pattern review, and the profile Codex review. Fix concrete P0-P2 findings and repeat only invalidated checks/review once.

## Acceptance rubric

- MUST: Stable exact Ghostty terminal UUID remains the activation target after title changes and cannot confuse duplicate titles. Proof: unit tests plus live macOS smoke when safe.
- MUST: One current registration per source; replacement invalidates old ownership handles; old-handle cleanup cannot delete replacement. Proof: store tests.
- MUST: Closed/missing terminals and unsupported provider/platform emit actionable `source.display.unavailable`-style diagnostics without changing successful tmux focus into focus failure. Proof: resolver/runtime/companion tests.
- MUST: Registered activation precedes existing caller and companion command fallbacks; configs containing only activation commands retain behavior. Proof: fallback tests.
- MUST: Existing registration JSON without display identity continues to deserialize or a tested migration is explicitly documented. Proof: fixture/serde test.
- MUST: CLI can capture/register, list/inspect health in JSON, and unregister only with matching handle. Proof: CLI tests and help inspection.
- MUST: No production target lookup uses title substring, terminal tty, or terminal pid. Proof: diff inspection and provider tests.
- MUST: Required focused commands and broad gates pass or have explicit environment evidence; HK sync/ready/handoff complete before PR.
- QUALITY: `alex-ai-testing-core` verifies behavior-first tests at the correct seams.
- QUALITY: `alex-ai-architecture-polish-review` verifies one clear registry owner, provider boundary, lifecycle, and deletion/test seams.
- QUALITY: `alex-ai-avoiding-ai-antipatterns` checks fallback observability and unnecessary abstractions/shims.
- QUALITY: `spec-sync` verifies ADR/SPEC/interface alignment.
- QUALITY: `alex-ai-writing-core` checks operator and ADR prose.
- QUALITY: `alex-ai-bash-core` checks the macOS smoke script if added.

Verdict: every MUST and selected QUALITY criterion passes; no averaging. A live smoke may be environment-blocked only with a recorded safe reason and deterministic provider/CLI proof; no operator terminal or default tmux server may be disturbed.

## Skill selection ledger

- APPLY:WORKFLOW `alex-ai-hk-plan` — large cross-module runtime/CLI/persistence change requires full HK lifecycle.
- APPLY:WORKFLOW `alex-ai-research` — trace existing caller, companion, registration, and ADR paths before implementation.
- APPLY:QUALITY `alex-ai-testing-core` — task explicitly requires seam-level unit and live-smoke proof.
- APPLY:QUALITY `alex-ai-architecture-polish-review` — new state ownership and provider boundary need architecture review.
- APPLY:QUALITY `alex-ai-avoiding-ai-antipatterns` — fallback and compatibility logic are prone to silent or duplicated behavior.
- APPLY:QUALITY `spec-sync` — public CLI and ADR 0004 contract change.
- APPLY:QUALITY `alex-ai-writing-core` — operator and ADR docs must be direct and accurate.
- APPLY:QUALITY `alex-ai-bash-core` — likely new macOS smoke script.
- SKIP `foreman-swift-overlay-ux` — no Swift overlay UI is in scope.
- SKIP `foreman-swift-overlay-validation` — requested overlay lane is validation evidence, not changed Swift UI harness ownership.
- SKIP `macos-app-design` / `macos-design-guidelines` — this is a command-line AppleScript adapter, not Mac UI design.
- SKIP `alex-ai-ratatui-tui` — focus behavior changes behind the TUI; rendering/input design does not change.
- SKIP `alex-ai-scope-based-hk-validation` — `hk-plan` already owns sizing and lifecycle; avoid duplicate workflow policy.

## Worker launch contract

Provider/model: `openai-codex/gpt-5.6-sol:high`
Reason: hard cross-module Rust persistence, CLI, runtime, companion, macOS adapter, docs, and smoke work.
Availability: inherited worker mapping is `openai-codex/gpt-5.6-sol`; exact model row and live sentinel probe passed on 2026-07-14.
Budget: maxRuntimeMs 7,200,000; 96 turns + 16 grace; tools 240 soft / 320 hard with all tools blocked after hard; control enabled, needs-attention after 300,000ms.
Escalation: stop for architecture/scope/safety decisions not resolved here; one fresh exact-model follow-up at most for bounded fixes; never resume/revive a model-pinned worker.

## Decisions and spec reflection
- Keep display identity machine-local in a source-owned registry; preserve source-host displayActivation JSON and add callerDisplayActivation; bound caller activation fallback to two seconds; split registry, Ghostty, command, and resolver seams.
  - Spec: updated: Spec/docs updated or verified.; refs: SPEC.md

## Learning
- None recorded.

## Gaps
- mise run validate-macos-overlay-change passed all five Rust control API tests, then local SwiftPM failed before branch code with no such module XCTest. No Swift files changed; CI must provide the overlay lane evidence.
- mise run verify passed the full 375-test Rust unit suite and subsequent integration lanes until runtime_dashboard::interactive_binary_popup_focus_action_exits_after_success. The test waited for a pane title containing `foreman`, but this required isolated worktree is named `ghostty-display-identity`; the captured UI was healthy and showed that worktree title. This is a worktree-path fixture assumption, not changed focus behavior.
- The opt-in live Ghostty capture -> rename -> exact focus smoke was not run because this noninteractive worker shell is not an explicitly authorized Ghostty terminal/TCC context. The script is syntax-checked, uses an isolated named tmux server, and its callerDisplayActivation assertion was verified against an isolated local focus reproduction. A human/CI Mac Ghostty session must run FOREMAN_GHOSTTY_DISPLAY_SMOKE=1.

## Validation evidence
- `sh -c 'cargo fmt --all -- --check && cargo test --lib source_display --quiet && cargo test --lib source_snapshots --quiet && cargo test --lib source_companion --quiet && cargo test --lib runtime --quiet && cargo test --lib services::control_api --quiet && cargo test --test source_display_registration --quiet && cargo clippy --all-targets --all-features -- -D warnings && sh -n scripts/smoke-ghostty-display-registration.sh && grep -q callerDisplayActivation scripts/smoke-ghostty-display-registration.sh'`: pass (exit 0) — validates: Final focused proof after correcting the live-smoke activation field. — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Final repository gate after all review fixes; known aggregation timing flake will be isolated if it recurs. — `<local HK state not exported>`
- `cargo test --lib services::extensions::tests -- --test-threads=1`: pass (exit 0) — validates: The only final fast-gate failures are shared process-environment extension tests; run them serially to avoid their global environment mutex race and poison cascade. — `<local HK state not exported>`
- `mise run validate-macos-overlay-change`: fail (exit 1) — attempted to validate: Required macOS validation lane for display/control behavior; record local XCTest availability if blocked. — `<local HK state not exported>`
- `mise run verify`: fail (exit 101) — attempted to validate: Required final full validation after focused proof and review fixes; classify known host-only test or Docker blockers explicitly. — `<local HK state not exported>`

## Readiness
- context: info — context recorded
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: pass — validation evidence with rationale recorded
- review: pass — external-enough review recorded
- profile-check:fast-gate: pass — validation dangerously skipped: fast-gate; reason: mise run check passed format, clippy, typecheck, 368 Rust tests, and the aggregation timing test; it failed only extension tests whose shared process-environment assertion poisoned their mutex under parallel execution.; mitigation: All issue-34 focused tests and clippy pass; the exact extension test module passes 8/8 serially; prior runs reproduced broad-suite environment/timing failures on clean main. CI must run the full gate in a clean runner.
- profile-review:codex-review: pass — required profile review recorded: codex-review (matched scripts/smoke-ghostty-display-registration.sh)

## Review
- subagent / reviewer-fresh-context [architecture-polish-review]: Second-pass review graded production behavior and architecture A with no P0/P1 findings. Its sole P2 found the live smoke reading displayActivation instead of callerDisplayActivation; commit ac20727 corrected that exact field and shell syntax plus focused smoke assertions passed. paths: scripts/smoke-ghostty-display-registration.sh. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Fresh-context final review covered the complete origin/main diff, correctness, compatibility, lifecycle, CLI, tests, and docs. It found no P0/P1 and graded production behavior A. The only P2 was the smoke field name; commit ac20727 fixed it and the invalidated shell/static checks pass. paths: scripts/smoke-ghostty-display-registration.sh. [accepted]

## Dangerous skips
- validation: fast-gate — reason: mise run check passed format, clippy, typecheck, 368 Rust tests, and the aggregation timing test; it failed only extension tests whose shared process-environment assertion poisoned their mutex under parallel execution.; mitigation: All issue-34 focused tests and clippy pass; the exact extension test module passes 8/8 serially; prior runs reproduced broad-suite environment/timing failures on clean main. CI must run the full gate in a clean runner.
- validation: heavy-gate — reason: mise run verify passed the full unit suite and many integration lanes, then one runtime-dashboard fixture timed out waiting for the canonical checkout title `foreman`; the isolated worktree correctly rendered `ghostty-display-identity`.; mitigation: Focused runtime/display tests, subprocess registration tests, clippy, and source action serialization pass; full unit suite passed 375/375 inside verify; CI in a canonical checkout must complete the remaining integration/Docker lanes.
