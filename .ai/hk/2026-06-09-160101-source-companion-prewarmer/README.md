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
- Implementation planning for PR #27 final 'just works' slice: use foreman-root profile. Current system map covers tmux-runtime-effects and macos-overlay-control but does not yet have a source-companion component/invariant; for this slice treat source companion as a tmux-runtime-effects subcomponent and add durable invariants in AGENTS/docs/ADR. Suggested future HK system-map invariant: source identity/actions remain source-scoped; companion source-local side effects run on the companion host; live transport failure degrades to snapshot/local rows; send requires explicit trusted boundary.
- User privacy/public-repo constraint for source companion happy path: built-in command names, docs, tests, and validation artifacts must stay generic. Avoid company-specific hostnames, Discord infrastructure names, or personal workflow details. Use generic terms like remote dev host, SSH host, laptop/workstation, and optionally Coder only as a vendor-neutral platform example. Prefer a generic command name over connect-coder.

## Plan
- Implement the first ADR 0004 slices: source snapshot store, fixture-backed freshness/diagnostic tests, explicit companion/prewarmer commands, runtime read-only snapshot loading behind opt-in config, and real Mac↔Coder validation using the available Coder SSH connection. Preserve one-shot SSH as fallback and keep popup key-latency budgets covered.

## Decisions and spec reflection
- Implemented the first ADR 0004 slice as a read-only snapshot source, not a daemon or reverse tunnel: SourceSnapshotStore owns atomic JSON snapshot lifecycle; sources snapshot writes a source-local AgentsResponse envelope; snapshot source config lets another Foreman instance render cached rows; live actions still require SSH/future companion transport.
- Expanded PR #27 from Phase 1 snapshot foundation to the full Phase 2-6 source companion definition of done in one PR: prewarmer loop, companion registration, reverse/live transport, live focus/send action transport, and display activation parity. Validation smokes should move from shell glue toward a Python integration harness.
- PR #27 now implements the source companion architecture through the code seams for Phase 2-6: prewarmer loop, registration metadata, companion live transport, gated focus/send actions, and activation-command configuration. Live Coder reverse-actions validation is not complete because the current Coder SSH proxy accepts ssh -R but does not expose a reachable forwarded port; this is documented as a transport/environment blocker rather than a protocol blocker.
- Diagnosed the apparent Coder ssh -R failure as a validation harness bug, not a Coder proxy/product transport blocker. Opening and closing the forwarded companion port without sending JSON could consume/stall the first companion request; probing with a valid companion request makes live Coder→Mac inventory/focus/send pass.
- Adopt A-lite design for PR #27 final slice: add foreman companion connect-coder as a thin Mac-side supervisor over existing companion server, ssh -R tunnel, remote Coder source upsert, snapshot/registration heartbeat, and companion-side Mac activation. Keep split commands as debug substrate; defer config-first daemon/launchd/relay to follow-up.
- Rename planned happy-path command from connect-coder to a generic remote-host command (preferred: foreman companion connect-ssh <host>, with any Coder-specific usage limited to examples/local validation flags). This keeps PR #27 public-facing and avoids encoding Alex/Discord infrastructure in CLI/docs.
  - Spec: updated: Spec/docs updated or verified.; refs: docs/operator-guide.md, docs/workflows.md
  - Spec: updated: Spec/docs updated or verified.; refs: docs/decisions/0004-source-companion-relay.md
  - Spec: updated: Spec/docs updated or verified.; refs: docs/decisions/0004-source-companion-relay.md, docs/operator-guide.md
  - Spec: updated: Spec/docs updated or verified.; refs: docs/decisions/0004-source-companion-relay.md, docs/operator-guide.md
  - Spec: updated: Spec/docs updated or verified.; refs: docs/decisions/0004-source-companion-relay.md, docs/operator-guide.md
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
- `scripts/source_companion_live_smoke.py reverse-actions --artifact-dir .ai/validation/source-companion/reverse-actions --install-coder --json --no-build`: pass (exit 0) — validates: Diagnosed reverse tunnel issue: fixed harness readiness probe to send valid companion JSON-line request; live Coder→Mac reverse tunnel inventory/focus/send now passes. — `<local HK state not exported>`
- `cargo test --lib source --quiet`: pass (exit 0) — validates: Focused source tests after reverse-action harness fix. — `<local HK state not exported>`
- `uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py .`: pass (exit 0) — validates: Docs verifier after recording source companion reverse-tunnel diagnosis and AGENTS gotcha. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after reverse-tunnel diagnosis fix and documentation update. — `<local HK state not exported>`
- `cargo test --lib source_companion --quiet`: pass (exit 0) — validates: Companion protocol and companion-side activation tests — `<local HK state not exported>`
- `cargo test --lib --quiet`: pass (exit 0) — validates: Full library tests after source companion connect-ssh — `<local HK state not exported>`
- `uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py .`: pass (exit 0) — validates: Docs verify after generic source companion docs — `<local HK state not exported>`
- `scripts/source_companion_live_smoke.py companion-local --artifact-dir .ai/validation/source-companion/companion-local --json --no-build`: pass (exit 0) — validates: Local companion JSON-line smoke with generic workstation source — `<local HK state not exported>`
- `scripts/smoke-popup-key-latency.sh`: pass (exit 0) — validates: Popup key latency remains bounded after connect-ssh changes — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py remote-snapshot --artifact-dir .ai/validation/source-companion/remote-snapshot --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --json --no-build'`: pass (exit 0) — validates: Live remote snapshot smoke via generic remote-host harness — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py connect-ssh --artifact-dir .ai/validation/source-companion/connect-ssh --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --json --no-build'`: pass (exit 0) — validates: Live connect-ssh smoke proves show focus send and source-local activation — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Final quality gate for connect-ssh source companion happy path — `<local HK state not exported>`
- `cargo test --lib companion_activation --quiet`: pass (exit 0) — validates: Activation timeout and connect-ssh supervisor tests — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py connect-ssh --artifact-dir .ai/validation/source-companion/connect-ssh --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --json --no-build'`: fail (exit 1) — attempted to validate: Connect-ssh smoke after readiness JSON and activation timeout polish — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py connect-ssh --artifact-dir .ai/validation/source-companion/connect-ssh --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --json --no-build'`: pass (exit 0) — validates: Connect-ssh smoke after readiness JSON and activation timeout polish — `<local HK state not exported>`
- `cargo test --lib --quiet`: pass (exit 0) — validates: Full library tests after isolated source companion harness — `<local HK state not exported>`
- `scripts/source_companion_live_smoke.py companion-local --artifact-dir .ai/validation/source-companion/companion-local --json --no-build`: pass (exit 0) — validates: Local companion smoke uses isolated tmux socket — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py remote-snapshot --artifact-dir .ai/validation/source-companion/remote-snapshot --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --json --no-build'`: pass (exit 0) — validates: Live remote snapshot smoke uses isolated workstation tmux socket — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py reverse-actions --artifact-dir .ai/validation/source-companion/reverse-actions --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --json --no-build'`: pass (exit 0) — validates: Live reverse-actions smoke uses isolated workstation tmux socket — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py connect-ssh --artifact-dir .ai/validation/source-companion/connect-ssh --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --json --no-build'`: pass (exit 0) — validates: Live connect-ssh smoke proves show focus send activation on isolated tmux — `<local HK state not exported>`
- `scripts/smoke-popup-key-latency.sh`: pass (exit 0) — validates: Popup key latency remains bounded after isolated harness fix — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Final quality gate after connect-ssh isolated tmux fix — `<local HK state not exported>`
- `uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py .`: pass (exit 0) — validates: Docs verify after source companion isolated tmux docs — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Final quality gate after connect-ssh trust and isolated tmux fixes — `<local HK state not exported>`
- `scripts/source_companion_live_smoke.py companion-local --artifact-dir .ai/validation/source-companion/companion-local --json --no-build`: pass (exit 0) — validates: Local companion smoke with token and isolated tmux socket — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py reverse-actions --artifact-dir .ai/validation/source-companion/reverse-actions --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --json --no-build'`: pass (exit 0) — validates: Live reverse-actions smoke with token and isolated tmux socket — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py connect-ssh --artifact-dir .ai/validation/source-companion/connect-ssh --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --json --no-build'`: pass (exit 0) — validates: Live connect-ssh smoke with token, show/focus/send/activation, and isolated tmux — `<local HK state not exported>`
- `bash -lc 'READY=$(mktemp); ./target/debug/foreman companion serve --bind 127.0.0.1:0 --token [REDACTED] --max-requests 1 --ready-file "$READY" --json >/tmp/foreman-probe-serve.out 2>/tmp/foreman-probe-serve.err & PID=$!; for i in $(seq 1 100); do [ -s "$READY" ] && break; sleep 0.05; done; EP=$(cat "$READY"); ./target/debug/foreman companion probe --endpoint "$EP" --token [REDACTED] --json; wait $PID; rm -f "$READY"'`: pass (exit 0) — validates: Foreman-native companion probe command works against local companion server — `<local HK state not exported>`
- `bash -lc 'scripts/source_companion_live_smoke.py connect-ssh --artifact-dir .ai/validation/source-companion/connect-ssh --remote-host "$FOREMAN_REMOTE_DEV_HOST" --remote-foreman "$FOREMAN_REMOTE_FOREMAN" --install-remote --json --no-build'`: pass (exit 0) — validates: connect-ssh smoke using Foreman-native remote companion probe and isolated tmux — `<local HK state not exported>`
- `mise run validate-macos-overlay-change`: pass (exit 0) — validates: macOS overlay parity lane after source companion/control API changes — `<local HK state not exported>`
- `cargo test --lib --quiet`: pass (exit 0) — validates: Full Rust library tests after replacing remote Python readiness probe with Foreman-native companion probe — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Quality gate after Foreman-native companion probe and macOS overlay validation — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after Foreman-native remote companion probe — `<local HK state not exported>`
- `mise run validate-macos-overlay-change`: pass (exit 0) — validates: macOS overlay required lane validates source-aware Control API and native app parity — `<local HK state not exported>`

## Readiness
- context: info — context recorded
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: pass — validation evidence with rationale recorded
- review: pass — external-enough review recorded
- profile-check:fast-gate: pass — required profile check recorded: fast-gate (matched AGENTS.md, docs/decisions/0004-source-companion-relay.md, docs/operator-guide.md, +11 more)
- profile-check:macos-overlay-required-lane: pass — required profile check recorded: macos-overlay-required-lane (matched src/services/control_api.rs)
- profile-review:codex-review: pass — required profile review recorded: codex-review (matched scripts/smoke-source-snapshot-remote.sh, scripts/source_companion_live_smoke.py, src/cli.rs, +7 more)

## Review
- subagent / reviewer: Reviewed source snapshot/prewarmer implementation. Fixed blockers: sources doctor now preserves successful snapshot warm/stale diagnostics; runtime aggregation carries successful snapshot diagnostics into operator UI diagnostics; source summary staleness reflects snapshot warm/stale diagnostics; live Coder smoke now asserts both local and Mac rows. paths: src/source_snapshots.rs, src/sources.rs, src/cli.rs, +4 more. [accepted]
- subagent / reviewer: Reviewed regenerated UX artifacts and live Coder snapshot smoke artifact as validation outputs from passing verify-ux and scripts/smoke-source-snapshot-coder.sh. paths: .ai/validation/ux/foreman-ux-diagnostic.gif, .ai/validation/ux/foreman-ux-flash.png, .ai/validation/ux/foreman-ux-gruvbox.png, +5 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Codex-review equivalent coverage for snapshot source implementation: checked source snapshot store, source config/provider integration, CLI commands, runtime diagnostics propagation, and live smoke script. Review blockers were fixed before this record. paths: src/source_snapshots.rs, src/sources.rs, src/cli.rs, +3 more. [accepted]
- subagent / reviewer [codex-review]: Reviewed expanded PR #27 source companion implementation. Blockers found: companion all-panes propagation and blocking companion socket reads. Both were fixed: CompanionSource now carries include_non_agents/all_panes into the protocol request, companion server sets per-connection read/write timeouts. Follow-ups noted: trust UX for allow_send/token, Coder proxy reverse-forward limitation, and fallback error detail. paths: src/source_companion.rs, src/source_snapshots.rs, src/sources.rs, +6 more. [accepted]
- subagent / reviewer: Targeted review plus fixes for source companion Phase 2-6 work. Review-only subagent found two blockers; implementation now addresses both and validation passed afterwards. paths: src/source_companion.rs, src/source_snapshots.rs, src/sources.rs, +7 more. [accepted]
- subagent / reviewer [codex-review]: Reviewed module export change for source_companion; src/lib.rs only exposes the new source_companion module used by cli/sources tests. paths: src/lib.rs. [accepted]
- subagent / reviewer: Reviewed src/lib.rs module export for source companion. paths: src/lib.rs. [accepted]
- subagent / reviewer: Reviewed reverse-tunnel diagnosis fix: harness now validates readiness with a real JSON-line companion request instead of half-opening the port; docs and AGENTS capture the gotcha; live Coder→Mac inventory/focus/send validation passes. paths: scripts/source_companion_live_smoke.py, docs/decisions/0004-source-companion-relay.md, docs/operator-guide.md, +2 more. [accepted]
- subagent / reviewer [codex-review]: Codex-review coverage for reverse-tunnel diagnosis follow-up: reviewed harness probe fix, docs, AGENTS gotcha, and validation evidence. paths: scripts/source_companion_live_smoke.py, docs/decisions/0004-source-companion-relay.md, docs/operator-guide.md, +2 more. [accepted]
- subagent / architecture-polish-review: Architecture polish review found lifecycle, activation timeout, JSON identity, token trust, and isolated-validation issues. Addressed: added child guard/remote command timeouts, bounded activation timeout, stdout readiness with source label, token requirement for allow-send manual companions, input validation, isolated tmux harness, docs updates, and .pi/state ignore. Remaining deferred: deeper signal handling and Foreman-native remote probe beyond python3. paths: .ai/validation/source-aggregation/source-companion-phase6-smoke.md, .ai/validation/ux/foreman-ux-diagnostic.gif, .ai/validation/ux/foreman-ux-flash.png, +23 more. [accepted]
- subagent / codex-review: Codex review findings addressed: allow-send now requires token and docs wire token; ::1 no longer accepted until IPv6 formatting is supported; connect-ssh validates SSH-ish inputs; child guard added for cleanup on unwind; readiness JSON goes to stdout with configured label; .pi/state ignored and removed. Remaining deferred: robust SIGTERM trap and Foreman-native probe replacing remote python3. paths: .ai/validation/source-aggregation/source-companion-phase6-smoke.md, .ai/validation/ux/foreman-ux-diagnostic.gif, .ai/validation/ux/foreman-ux-flash.png, +23 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Fresh reviewer checked the Foreman-native companion probe diff. No blockers. Confirmed  is wired to CompanionClient, connect-ssh now probes through remote Foreman instead of embedded Python, smoke harness uses the remote Foreman probe, docs match, and local probe/cargo checks passed in review. paths: docs/operator-guide.md, scripts/source_companion_live_smoke.py, src/cli.rs, +1 more. [accepted]
- subagent / codex-review [codex-review]: Earlier codex/architecture review covered these PR paths; actionable findings were addressed before the final companion-probe patch. No additional changes were made to these paths in the Foreman-native probe follow-up. paths: scripts/smoke-source-snapshot-remote.sh, src/lib.rs, src/services/control_api.rs. [accepted]
