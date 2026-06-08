# HK export: `2026-06-04-135616-source-aggregation-ssh-coder`

This directory is a generated review/handoff package from the Harness Kit ledger. Do not hand-edit it; update HK with `hk plan`, `hk decide`, `hk validate`, `hk review add`, and `hk sync`, then regenerate.

## Freshness
Validate this export against local HK state with:

```bash
hk export --format handoff-dir --output .ai/hk/2026-06-04-135616-source-aggregation-ssh-coder --target . --check
```

Historical hand-authored slice plans live under `.ai/plans/`; new Harness Toolkit repo work should use HK and generated `.ai/hk/` exports.

## Handoff

## Summary
- Work: `2026-06-04-135616-source-aggregation-ssh-coder`
- Branch: `feature/source-aggregation-ssh-coder`

## Context
- None recorded.

## Plan
- Implement source aggregation, SSH Coder sources, and source-aware TUI/macOS overlay parity

## Decisions and spec reflection
- Implemented a one-shot source aggregation foundation, but final architecture review found full ADR DoD is not met because interactive Ratatui runtime still uses local tmux inventory instead of SourceAggregator. Treat remaining work as follow-up before merge or narrow branch scope to control API + macOS groundwork.
- Continue source-aggregation work on Design A: app-domain source identity deepening for full Ratatui/tmux-popup and macOS overlay parity. Do not use the runtime adapter shim/control-API-as-runtime-model design except as disposable spike. Keep the original validation ladder, but reorder it around app identity, runtime aggregation, source-routed effects, TUI rendering, and final Coder/manual smokes.
  - Spec: updated: Spec/docs updated or verified.; refs: docs/decisions/0002-source-aggregation-and-remote-ssh.md
  - Spec: updated: Spec/docs updated or verified.; refs: docs/decisions/0002-source-aggregation-and-remote-ssh.md

## Learning
- None recorded.

## Gaps
- None recorded.

## Validation evidence
- `cargo test sources --lib`: pass (exit 0) — validates: Source aggregation, source identity, and tmux server-name Rust tests — `<local HK state not exported>`
- `bash -lc 'cargo test control_api --lib && cargo test tmux --lib'`: pass (exit 0) — validates: Control API and tmux target tests after source-aware schema — `<local HK state not exported>`
- `swift test --package-path apps/macos-overlay`: pass (exit 0) — validates: Source-aware macOS overlay model and client tests — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Fast gate after one-shot source aggregation implementation — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Fast gate after source aggregation implementation and clippy fixes — `<local HK state not exported>`
- `mise run validate-macos-overlay-change`: fail (exit 1) — attempted to validate: Source-aware overlay behavior and fixtures changed — `<local HK state not exported>`
- `mise run validate-macos-overlay-change`: pass (exit 0) — validates: Source-aware overlay behavior and real tmux control API smoke — `<local HK state not exported>`
- `bash -lc 'cargo test sources --lib && cargo test control_api --lib && cargo test cli::tests --lib && cargo test tmux --lib'`: pass (exit 0) — validates: Rust source aggregation unit, control API, CLI, and tmux target coverage — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after source aggregation implementation; previous notification test flake passed standalone — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after Rust source aggregation core changes — `<local HK state not exported>`
- `bash -lc 'set -euo pipefail
name=foreman-smoke-$$
tmux -L "$name" new-session -d -s spike "sh -lc sleep\\ 30"
trap "tmux -L $name kill-server >/dev/null 2>&1 || true" EXIT
target/debug/foreman --tmux-server-name "$name" agents --json --all-panes | jq -e ".entries[0].sourcePaneId == \"source:local:pane:%0\""
tmp=$(mktemp -d)
FOREMAN_CONFIG_HOME="$tmp" target/debug/foreman sources add ssh coder-dev-gpu-1 --host coder.alex-furrier-dev-gpu-1 --foreman /home/discord/.cargo/bin/foreman --tmux-server-name user --label "Coder dev-gpu-1" --json >/dev/null
FOREMAN_CONFIG_HOME="$tmp" target/debug/foreman agents --json --sources all --all-panes | jq -e ".sources | length >= 2" >/dev/null
FOREMAN_CONFIG_HOME="$tmp" target/debug/foreman agents --json --sources all --all-panes | jq -e ".partialFailureCount >= 0" >/dev/null
'`: pass (exit 0) — validates: Manual local tmux server-name and Coder SSH source smoke for source aggregation — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after extension routing and SSH timeout hardening — `<local HK state not exported>`
- `cargo test --lib`: pass (exit 0) — validates: Design A app-domain source identity, runtime source aggregation, source routed focus/send, source diagnostics, and parallel source fanout — `<local HK state not exported>`
- `swift test --package-path apps/macos-overlay`: pass (exit 0) — validates: Swift overlay regression after Rust source identity/control API changes — `<local HK state not exported>`
- `scripts/smoke-control-api-tmux.sh`: pass (exit 0) — validates: Control API tmux smoke after source-aware app identity changes — `<local HK state not exported>`
- `mise run validate-macos-overlay-change`: pass (exit 0) — validates: Required overlay lane after source-aware control API/runtime model changes — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Fast gate after Design A source-aware TUI/runtime parity implementation — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after fixing integration test source-key compile fallout — `<local HK state not exported>`
- `bash -lc '
tmp=$(mktemp -d)
cat >$tmp/ssh <<'"'"'SH'"'"'
#!/bin/sh
cat <<'"'"'JSON'"'"'
{"schemaVersion":1,"generatedAtUnixMs":1,"inventory":{"totalSessions":1,"totalWindows":1,"totalPanes":1,"visibleSessions":1,"visibleWindows":1,"visiblePanes":1},"entries":[{"id":"source:local:pane:%42","sourcePaneId":"source:local:pane:%42","sourceId":"local","sourceLabel":"RemoteFixture","sourceKind":"ssh","paneId":"%42","sessionId":"$remote","sessionName":"remote","windowId":"@remote","windowName":"agents","title":"remote-pi","navigationTitle":"remote-pi","harness":"pi","harnessLabel":"Pi","status":"working","statusLabel":"WORKING","statusSource":"compatibility","integrationMode":"compatibility","isAgent":true,"currentCommand":"pi","runtimeCommand":"pi","workingDir":"/tmp/remote","linkedRepository":null,"workspaceName":"remote","preview":"remote working","previewProvenance":"captured","activityScore":5,"statusRank":2,"lastActivityUnixMs":1,"lastStatusChangeUnixMs":1,"activeRunCount":null,"pullRequest":null,"extensionCards":[]}],"diagnostics":[],"sources":[],"sourceDiagnostics":[],"partialFailureCount":0}
JSON
SH
chmod +x $tmp/ssh
cat >$tmp/config.toml <<EOF
[sources]
default_scope = "all"
query_timeout_ms = 1000

[sources.coder]
kind = "ssh"
label = "Coder"
host = "fake"
ssh = "$tmp/ssh"
foreman = "foreman"
EOF
./target/debug/foreman --config-file $tmp/config.toml --log-dir $tmp/log --sources all --bootstrap-only --no-notify
rm -rf $tmp
'`: pass (exit 0) — validates: Bootstrap/popup runtime source aggregation smoke with fake SSH source and all-sources scope — `<local HK state not exported>`
- `bash -lc '
tmp=$(mktemp -d)
cat >$tmp/config.toml <<EOF
[sources]
default_scope = "all"
query_timeout_ms = 3000

[sources.coder-dev-gpu-1]
kind = "ssh"
label = "Coder dev GPU"
host = "coder.alex-furrier-dev-gpu-1"
tmux_server_name = "user"
foreman = "/home/discord/.cargo/bin/foreman"
EOF
./target/debug/foreman --config-file $tmp/config.toml --sources all agents --json | tee $tmp/out.json
rg "sourceDiagnostics|unexpected argument|coder-dev-gpu-1" $tmp/out.json
rm -rf $tmp
'`: pass (exit 0) — validates: Manual Coder source diagnostic remains graceful when remote Foreman is older than branch — `<local HK state not exported>`
- `cargo test --lib`: pass (exit 0) — validates: Focused Rust after source-scoped notification cooldowns and remote extension routing — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after source-scoped cooldowns/schema/extension follow-ups — `<local HK state not exported>`
- `cargo test --lib`: pass (exit 0) — validates: Focused Rust after local-only gating for destructive actions on remote selections — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after destructive-action source gating — `<local HK state not exported>`
- `mise run verify-native`: pass (exit 0) — validates: Strict native harness regression after source-key identity changes touched native integration test helpers — `<local HK state not exported>`
- `swift test --package-path apps/macos-overlay`: pass (exit 0) — validates: Swift overlay after control API schema version alignment to v2 — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Final fast gate after schema v2 alignment and destructive-action gating — `<local HK state not exported>`
- `cargo test --lib`: pass (exit 0) — validates: Focused Rust after source-scoped notification request identity and --source/cache fixes — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after source filter, startup cache scope skip, and notification identity fixes — `<local HK state not exported>`
- `mise run validate-macos-overlay-change`: fail (exit 1) — attempted to validate: Final macOS overlay lane after schema v2/source identity follow-ups — `<local HK state not exported>`
- `scripts/smoke-control-api-tmux.sh`: pass (exit 0) — validates: Control API smoke after schema v2 update — `<local HK state not exported>`
- `mise run validate-macos-overlay-change`: pass (exit 0) — validates: Final macOS overlay lane after schema v2 smoke update — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after final smoke script schema v2 update — `<local HK state not exported>`
- `bash -lc '
set -euo pipefail
REMOTE=coder.alex-furrier-dev-gpu-1
./target/debug/foreman sources doctor coder-dev-gpu-1 --json | jq -e ".ok == true and .schemaVersion == 2" >/dev/null
./target/debug/foreman agents --sources all --json > /tmp/foreman-hk-live-coder-agents.json
jq -e "(.schemaVersion == 2) and ([.entries[].sourceId] | index(\"local\") != null) and ([.entries[].sourceId] | index(\"coder-dev-gpu-1\") != null)" /tmp/foreman-hk-live-coder-agents.json >/dev/null
PANE=$(jq -r ".entries[] | select(.sourceId == \"coder-dev-gpu-1\") | .paneId" /tmp/foreman-hk-live-coder-agents.json | head -n 1)
./target/debug/foreman --source coder-dev-gpu-1 focus --pane "$PANE" --json | jq -e ".ok == true and .sourceId == \"coder-dev-gpu-1\"" >/dev/null
./target/debug/foreman --source coder-dev-gpu-1 extensions --pane "$PANE" --json | jq -e ".ok == true and .sourceId == \"coder-dev-gpu-1\" and .schemaVersion == 2" >/dev/null
ssh "$REMOTE" '"'"'rm -f /tmp/foreman-live-smoke.out; tmux kill-session -t foreman-live-smoke 2>/dev/null || true; tmux new-session -d -s foreman-live-smoke "sh -c '"'"'\\read line; echo FOREMAN_SMOKE:\$line > /tmp/foreman-live-smoke.out; sleep 10'"'"'\\"; tmux list-panes -t foreman-live-smoke -F "#{pane_id}"'"'"' > /tmp/foreman-live-smoke-pane.txt
SMOKE_PANE=$(tr -d "\r\n" < /tmp/foreman-live-smoke-pane.txt)
./target/debug/foreman --source coder-dev-gpu-1 send --pane "$SMOKE_PANE" --text "hello-from-hk-live-smoke" --json | jq -e ".ok == true and .sourceId == \"coder-dev-gpu-1\" and .bytesSent > 0" >/dev/null
ssh "$REMOTE" '"'"'grep -q "FOREMAN_SMOKE:hello-from-hk-live-smoke" /tmp/foreman-live-smoke.out; tmux kill-session -t foreman-live-smoke 2>/dev/null || true'"'"'
./target/debug/foreman --sources all --bootstrap-only --no-notify
'`: fail (exit 1) — attempted to validate: Live Coder source happy path after installing branch Foreman on Coder: persisted source doctors cleanly, all-source inventory includes local and coder, source-scoped focus/extensions/send route over SSH. — `<local HK state not exported>`
- `/tmp/foreman-live-coder-smoke.sh`: pass (exit 0) — validates: Live Coder source happy path after installing branch Foreman on Coder: persisted source doctors cleanly, all-source inventory includes local and coder, source-scoped focus/extensions/send route over SSH. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after increasing SSH source default timeout for real Coder extension latency and live Coder smoke. — `<local HK state not exported>`
- `bash -lc 'cargo test --lib && cargo test --test runtime_profiling -- --ignored'`: pass (exit 0) — validates: Source UX/perf hardening: source display config, staged all-source inventory refresh, source cache helpers, and SSH-vs-local runtime profiling smoke. — `<local HK state not exported>`
- `mise run verify-ux`: pass (exit 0) — validates: All-source popup UX/perf proof: verify-ux now includes fake SSH source profiling that compares all-source startup to local-only baseline and proves local rows render before slow SSH completes. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after source UX/perf hardening and runtime profiling updates. — `<local HK state not exported>`
- `bash -lc 'cargo test --lib && cargo test --test runtime_profiling -- --ignored && swift test --package-path apps/macos-overlay'`: pass (exit 0) — validates: Follow-up fixes after source UX/perf review: cached remote rows survive SSH refresh failure; control API and Swift overlay honor sourceShowLabel/sourceDisplayLabel. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after source cache fallback and source display fields for TUI/macOS overlay. — `<local HK state not exported>`
- `mise run validate-macos-overlay-change`: pass (exit 0) — validates: Required macOS overlay lane after sourceDisplayLabel/sourceShowLabel model and row badge behavior changes. — `<local HK state not exported>`
- `bash -lc 'cargo test --lib cli::tests::sources_add_ssh_persists_and_lists_source --quiet && cargo run --quiet -- sources doctor coder-dev-gpu-1 --json | jq -e ".tmuxEndpoint.kind == \"ssh\"" >/dev/null'`: pass (exit 0) — validates: Focused Rust after adding source doctor tmux endpoint summary for remote SSH endpoint mismatch diagnosis. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after source doctor tmux endpoint summary. — `<local HK state not exported>`
- `bash -lc 'cargo test --lib && cargo test --test runtime_profiling -- --ignored'`: pass (exit 0) — validates: Diagnose follow-up: re-enable popup startup cache for all-source mode and make Enter on popup session rows focus their actionable pane. — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Fast gate retry after diagnose fixes; previous invocation timed out while running full Rust tests after format/lint/typecheck passed. — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Fast gate after diagnose fixes; rerun after transient workspace_scope extension timeout passed in isolation. — `<local HK state not exported>`
- `bash -lc 'RUST_TEST_THREADS=1 mise run check'`: fail (exit 101) — attempted to validate: Fast gate after diagnose fixes with serialized Rust tests because codex_native tmux fixture is flaky under parallel test execution on this machine; failed fixture passed in isolation. — `<local HK state not exported>`
- `bash -lc 'cargo test --lib --quiet && cargo test --test runtime_profiling -- --ignored'`: pass (exit 0) — validates: Diagnose keypress lag: popup suppresses automatic PR/extension lookups during navigation and caps popup input poll timeout; validates unit and runtime profiling suites. — `<local HK state not exported>`
- `bash -lc 'cargo test --lib --quiet && cargo test --test runtime_profiling -- --ignored && scripts/smoke-popup-key-latency.sh'`: pass (exit 0) — validates: Popup input-first scheduling and idle-deferred all-source merge: unit suite, runtime profiling suite, and popup key-latency smoke. — `<local HK state not exported>`
- `scripts/smoke-popup-key-latency.sh`: fail (exit 1) — attempted to validate: Popup responsiveness proof: key-latency smoke compares local-only, all-source idle, and all-source refresh-overlap with deferred merge. — `<local HK state not exported>`
- `scripts/smoke-popup-key-latency.sh`: pass (exit 0) — validates: Popup responsiveness proof: key-latency smoke compares local-only, all-source idle, and all-source refresh-overlap with idle-deferred merge. — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Fast gate after popup input-first scheduling, idle-deferred source merges, key-latency smoke, and source jump activation config. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate retry after transient runtime_dashboard tmux fixture banner failures passed in isolation. — `<local HK state not exported>`
- `mise run verify-ux`: pass (exit 0) — validates: Full verify-ux after popup input-first/deferred merge scheduling changes. — `<local HK state not exported>`
- `bash -lc 'cargo test --lib config::tests::config_parsing_supports_source_jump_activate_command_alias --quiet && cargo test --test runtime_profiling popup_defers_all_source_merge_until_navigation_is_idle -- --ignored --nocapture && scripts/smoke-popup-key-latency.sh'`: pass (exit 0) — validates: Follow-up fixes from targeted review: source jump activate_command alias, enforcing popup key-latency smoke budgets, and focused config/smoke coverage. — `<local HK state not exported>`
- `mise run check`: pass (exit 0) — validates: Fast gate after source jump activate_command alias and enforced popup key-latency smoke budgets. — `<local HK state not exported>`
- `bash -lc 'uv run python /Users/alex.furrier/.pi/agent/skills/context-engineering-context-docs/scripts/docs_verify.py . && cargo test --lib --quiet && mise run check'`: pass (exit 0) — validates: Final PR cleanup: remove scratch root docs, update source operator docs/frontmatter/index, and keep CI-fixed remote PR lookup/test serialization changes covered. — `<local HK state not exported>`
- `mise run verify-ux --capture-only`: pass (exit 0) — validates: Wire popup key-latency smoke into verify-ux so source/popup keypress regressions are covered by the automated UX lane. — `<local HK state not exported>`

## Readiness
- context: info — no context recorded; okay for trivial work, add hk context if it prevents rediscovery
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: pass — validation evidence with rationale recorded
- review: pass — external-enough review recorded
- profile-check:fast-gate: pass — required profile check recorded: fast-gate (matched .mise/tasks/verify-ux, AGENTS.md, apps/macos-overlay/Fixtures/agents-sources.json, +51 more)
- profile-check:macos-overlay-required-lane: pass — required profile check recorded: macos-overlay-required-lane (matched apps/macos-overlay/Fixtures/agents-sources.json, apps/macos-overlay/Sources/ForemanOverlayCore/ForemanClient.swift, apps/macos-overlay/Sources/ForemanOverlayCore/Models.swift, +8 more)
- profile-check:native-harness-strict: pass — required profile check recorded: native-harness-strict (matched src/integrations/claude.rs, src/integrations/codex.rs, src/integrations/native.rs, +1 more)
- profile-review:codex-review: pass — required profile review recorded: codex-review (matched .mise/tasks/verify-ux, scripts/smoke-control-api-tmux.sh, scripts/smoke-popup-key-latency.sh, +35 more)

## Review
- subagent / reviewer-fresh-context [architecture-polish-review]: Architecture polish review graded the current one-shot source aggregation branch C+ and identified remaining gaps: interactive Ratatui runtime does not yet consume SourceAggregator, SSH fan-out is not parallel/stale-cache complete, ambiguous bare pane handling remains incomplete, and source validation/docs need additional hardening. P1/P2 follow-ups were partially addressed: SSH hard timeout, remote extension routing, local source config honor, and validation reruns. paths: .ai/validation/macos-overlay/app-bundle/launch.log, .ai/validation/macos-overlay/app-bundle/summary.md, .ai/validation/macos-overlay/gauntlet/fake-foreman-calls.log, +73 more. [accepted]
- subagent / architecture-polish-review [architecture-polish-review]: Final architecture polish review after Design A source identity/runtime parity pass graded branch A-. It found no remaining P0/P1 blockers after destructive tmux actions were gated local-only for remote selections. Remaining guidance is review packaging: curate generated validation artifacts and keep overlay evidence explicit. paths: .ai/validation/macos-overlay/app-bundle/launch.log, .ai/validation/macos-overlay/app-bundle/summary.md, .ai/validation/macos-overlay/gauntlet/fake-foreman-calls.log, +91 more. [accepted]
- subagent / codex-review [codex-review]: Codex review found remaining issues: --source ignored by interactive runtime, startup cache not source/scope-aware, TUI PR/extension caches keyed by workspace only, remote linked_repository metadata dropped in TUI aggregation, notification request identity still pane-id only, current_source_first unimplemented, and missing runtime-path tests. Will address high/medium source correctness issues before final readiness. paths: .ai/validation/macos-overlay/app-bundle/launch.log, .ai/validation/macos-overlay/app-bundle/summary.md, .ai/validation/macos-overlay/gauntlet/fake-foreman-calls.log, +91 more. [accepted]
- subagent / codex-review [codex-review]: Targeted follow-up addressed Codex findings: interactive --source is now stored in RuntimeConfig and runtime aggregation filters to that source; popup startup cache is skipped for all/source-scoped launches to avoid cross-scope stale panes; notification requests and cooldowns carry PaneKey/source-scoped identity; control API schema aligned to v2 and smoke script updated. Final fast gate and macOS overlay lane passed after changes. paths: scripts/smoke-control-api-tmux.sh, src/app/reducer.rs, src/cli.rs, +6 more. [accepted]
- subagent / validation-artifact-review: Reviewed generated macOS overlay validation artifacts from final required lane. These are expected output updates from validate-macos-overlay-change (app bundle, fake-Foreman gauntlet, headless snapshots/OCR, verify summary), not hand-authored product logic. paths: .ai/validation/macos-overlay/app-bundle/launch.log, .ai/validation/macos-overlay/app-bundle/summary.md, .ai/validation/macos-overlay/gauntlet/overlay.log, +3 more. [accepted]
- subagent / reviewer-fresh-context: Targeted review of SSH source timeout follow-up: default query timeout increased from 1500ms to 5000ms after live Coder extension lookup exceeded 1500ms; docs updated to match. Reviewer found no blockers; noted worst-case unreachable SSH source latency increases to ~5s but remains per-source configurable and fan-out is parallel. paths: docs/operator-guide.md, docs/decisions/0002-source-aggregation-and-remote-ssh.md, src/sources.rs. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Targeted codex-review follow-up for src/sources.rs timeout default change. Reviewed the 1500ms -> 5000ms default query_timeout_ms update after live Coder validation; no P0/P1 concerns found. Timeout remains configurable per SSH source and source fan-out is parallel. paths: src/sources.rs. [accepted]
- subagent / reviewer-fresh-context: Follow-up review after source UX/perf P1 fixes: cached remote rows now survive failed SSH refresh with diagnostics, and control API/Swift overlay honor sourceDisplayLabel/sourceShowLabel so local badges can be hidden and remote labels shown. Reviewer found no blockers or new P0/P1 regressions. paths: src/runtime.rs, src/services/control_api.rs, apps/macos-overlay/Sources/ForemanOverlayCore/Models.swift, +5 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Targeted codex-review follow-up for source UX/perf hardening: staged all-source local/cached render, source display config/labels, runtime profiling fake SSH baseline comparison, and cache fallback on SSH failure. Follow-up reviewer found prior P1s fixed and no new P0/P1 regressions. paths: src/app/fixtures.rs, src/app/state.rs, src/cli.rs, +5 more. [accepted]
- subagent / validation-artifact-and-plan-review: Reviewed generated validation artifacts and planning/context docs after source UX/perf hardening. macOS/UX artifacts were produced by passing validate-macos-overlay-change and verify-ux; goal files were Plannotator-gated; AGENTS/docs updates capture agreed source UX/perf and jump-to direction. paths: .ai/validation/macos-overlay/app-bundle/launch.log, .ai/validation/macos-overlay/app-bundle/summary.md, .ai/validation/macos-overlay/gauntlet/overlay.log, +42 more. [accepted]
- subagent / reviewer-fresh-context: Reviewed source doctor tmux endpoint summary follow-up: sources doctor now reports configured server/socket and notes that noninteractive SSH does not inherit attached terminal /private/tmp/tmux-503/default,10805,8, helping diagnose Coder tab vs SSH source endpoint mismatches. Focused CLI test and fast gate passed. paths: src/cli.rs. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Targeted codex-review follow-up for source doctor endpoint summary in src/cli.rs; change is diagnostic-only, reports configured tmux endpoint for local/SSH sources, and fast gate passed. paths: src/cli.rs. [accepted]
- subagent / reviewer: Targeted reviewer found two P1s for popup scheduling/jump follow-up: documented activate_command alias was not parsed and popup key-latency smoke did not enforce budgets. Both were fixed: SourceJumpConfig accepts activate_command alias with config test coverage, and smoke now fails on slow actions/render budget/all-source overhead/missing deferred apply. Focused validation ev_20260608_130141_143207 passed. paths: src/runtime.rs, src/sources.rs, src/config.rs, +4 more. [accepted]
- subagent / reviewer [codex-review]: Codex-review coverage for popup input-first scheduling, idle-deferred source merges, source jump activation alias, and enforced key-latency smoke. P1 findings from reviewer were fixed and focused validation passed. paths: src/runtime.rs, src/sources.rs, src/config.rs, +2 more. [accepted]
- subagent / validation-artifact-and-followup-review: Reviewed stale UX validation artifacts and prior source action/CLI paths after verify-ux and fast-gate reruns. UX artifacts regenerated by passing verify-ux ev_20260608_125414_255900; source action/CLI paths unchanged by latest popup scheduling fix except covered by fast-gate ev_20260608_130347_264593. paths: .ai/validation/ux/foreman-ux-diagnostic.gif, .ai/validation/ux/foreman-ux-flash.png, .ai/validation/ux/foreman-ux-gruvbox.png, +7 more. [accepted]
- subagent / validation-artifact-and-followup-review [codex-review]: Codex-review follow-up for stale src/app/action.rs and src/cli.rs coverage after latest sync; latest changes were in popup scheduling/jump config and fast gate passed. paths: src/app/action.rs, src/cli.rs. [accepted]
- subagent / context-docs-and-ci-cleanup: Reviewed final PR cleanup: removed scratch root/goal reports from commit, updated operator source config docs to match implemented display/jump fields, added ADR frontmatter/index entry, fixed remote PR lookup to skip non-local source paths, and serialized fake SSH tests to avoid CI races. paths: docs/AGENTS.md, docs/operator-guide.md, docs/decisions/0003-remote-jump-terminal-activation.md, +2 more. [accepted]
- subagent / context-docs-and-ci-cleanup [codex-review]: Addressed Codex P2 by skipping local PR lookup for non-local source entries; also stabilized source SSH unit tests after CI-only race failures. paths: src/cli.rs, src/sources.rs. [accepted]
- subagent / key-latency-validation-wiring: Reviewed automated popup key-latency coverage: verify-ux now invokes scripts/smoke-popup-key-latency.sh after runtime_profiling, AGENTS/docs explain when to run it, and validation passed with local-only/all-source-idle/overlap budgets enforced. paths: .mise/tasks/verify-ux, AGENTS.md, docs/workflows.md, +1 more. [accepted]
- subagent / key-latency-validation-wiring: Reviewed regenerated UX validation artifacts from verify-ux after wiring key-latency smoke; artifacts are expected output from passing validation. paths: .ai/validation/ux/foreman-ux-diagnostic.gif, .ai/validation/ux/foreman-ux-flash.png, .ai/validation/ux/foreman-ux-gruvbox.png, +4 more. [accepted]
- subagent / key-latency-validation-wiring [codex-review]: Codex-review follow-up for validation wiring only: verify-ux now runs existing popup key-latency smoke; no product behavior change. paths: .mise/tasks/verify-ux. [accepted]

## Sync exclusions
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
- .pi: agent-local-state
