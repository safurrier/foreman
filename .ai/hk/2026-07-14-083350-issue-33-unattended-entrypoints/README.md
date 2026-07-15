# HK export: `2026-07-14-083350-issue-33-unattended-entrypoints`

This directory is a generated review/handoff package from the Harness Kit ledger. Do not hand-edit it; update HK with `hk plan`, `hk decide`, `hk validate`, `hk review add`, and `hk sync`, then regenerate.

## Freshness
Validate this export against local HK state with:

```bash
hk export --format handoff-dir --output .ai/hk/2026-07-14-083350-issue-33-unattended-entrypoints --target . --check
```

Historical hand-authored slice plans live under `.ai/plans/`; new Harness Toolkit repo work should use HK and generated `.ai/hk/` exports.

## Handoff

## Summary
- Work: `2026-07-14-083350-issue-33-unattended-entrypoints`
- Branch: `feature/issue-33-unattended-entrypoints`

## Context
- Issue #33 entrypoints are thin orchestration: receipt JSON preserves the canonical doctor report, while log paths follow FOREMAN_LOG_DIR/XDG_STATE_HOME/default resolution without parsing log content.

## Plan
- Add versioned non-interactive .agents/setup and read-only .agents/resume entrypoints without duplicating setup, doctor, HK, or logging ownership. TDD with isolated command fakes and fixture checkouts covering setup success/repeat/failure/strict-doctor blocking plus resume clean/dirty/detached/missing optional tools. Document schema, commands, and exit semantics in the agent-facing workflow surface. Validate focused behavioral tests and shell lint, then run mise run check once stable; obtain required independent review, fix P0-P2 findings, sync, ready, and export a compact HK handoff before draft PR. Skill ledger: APPLY workflow hk-plan and supervised-dev-dev-kickoff; APPLY quality bash-core, testing-core, writing-core, avoiding-ai-antipatterns, architecture-polish-review; SKIP Swift overlay skills (no overlay paths), ratatui (no TUI code), spec-sync (no product invariant change expected). Worker preflight selected openai-codex/gpt-5.6-terra:medium after exact model listing and live WORKER_MODEL_OK probe. Runaway guard: 7200000ms, 96+16 turns, tools 240/320 block all, attention 300000ms.

## Decisions and spec reflection
- Use one JSON-only renderer so setup and resume share schema encoding while setup/doctor/HK/logging ownership stays with canonical commands.
  - Spec: not-needed: Spec/docs update not needed.

## Learning
- None recorded.

## Gaps
- Bounded follow-up is blocked by exact-model policy: original implementation worker was openai-codex/gpt-5.6-terra:medium with maxRuntimeMs=7200000, turn budget 96+16, tool budget 240/320 block-all, control attention=300000ms. On 2026-07-14, exact `pi --list-models openai-codex/gpt-5.6-terra` returned no matching model, so no follow-up worker was launched and no provider/model/budget substitution was made. Resume this deterministic session after that exact model is available; fix the recorded P1/P2 review findings, rerun invalidated focused checks and one review pass, then finish HK sync/ready/handoff and open the required draft PR.
- mise run check was attempted twice after the receipt changes. Both runs passed fmt, clippy, cargo check, and the newly wired 7-test Python suite. Run 1 failed only runtime_dashboard::interactive_binary_popup_focus_action_exits_after_success (tmux alternate-screen timing). Run 2 failed extensions::explicit_provider_env_overrides_repo_provider_with_same_id (2 vs 1), then six extensions tests poisoned on that failure. These unchanged Rust tests are outside issue #33; no unrelated test behavior was modified.

## Validation evidence
- `/usr/bin/python3 tests/agent_entrypoints_test.py`: pass (exit 0) — validates: Isolated fake-command behavioral tests prove unattended setup convergence/failures and read-only resume status variants without touching the live environment. — `<local HK state not exported>`
- `mise run check`: fail (exit 1) — attempted to validate: Final fast gate validates Rust checks plus the stable repository after adding unattended entrypoints and isolated behavioral coverage. — `<local HK state not exported>`
- `shellcheck --shell=bash .agents/setup .agents/resume`: pass (exit 0) — validates: ShellCheck validates strict Bash entrypoint syntax and portability diagnostics. — `<local HK state not exported>`
- `bash -lc '/usr/bin/python3 tests/agent_entrypoints_test.py && shellcheck --shell=bash .agents/setup .agents/resume'`: pass (exit 0) — validates: Final committed entrypoint content passes isolated command-fake behavior coverage and ShellCheck. — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Run Foreman's required stable pre-PR gate after focused unattended-entrypoint tests and ShellCheck pass; the earlier trust refusal did not execute the gate. — `<local HK state not exported>`
- `cargo test sources::tests::aggregator_queries_sources_in_parallel --all-features`: fail (exit 101) — attempted to validate: The sole fast-gate failure was an unrelated timing-sensitive source aggregation test; its isolated rerun passes without branch changes. — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Required stable pre-PR gate after rebase and focused entrypoint tests — `<local HK state not exported>`
- `bash -lc 'uv run python tests/agent_entrypoints_test.py && bash -n .agents/setup .agents/resume && shellcheck --shell=bash .agents/setup .agents/resume && uv run python -m py_compile .agents/entrypoint-report.py tests/agent_entrypoints_test.py && git diff --check'`: pass (exit 0) — validates: Review follow-up: isolated behavioral coverage proves portable structured receipts, canonical log discovery, visible schema errors, actionable diagnostics, read-only resume, and HK stderr preservation. — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Final canonical check ran the new Python behavioral suite successfully, then failed only in unchanged runtime_dashboard::interactive_binary_popup_focus_action_exits_after_success tmux timing/alternate-screen assertion. — `<local HK state not exported>`
- `bash -lc '/usr/bin/python3 tests/agent_entrypoints_test.py && uvx ruff check .agents/entrypoint-report.py tests/agent_entrypoints_test.py && bash -n .agents/setup .agents/resume && shellcheck --shell=bash .agents/setup .agents/resume && git diff --check'`: pass (exit 0) — validates: Nine isolated entrypoint behavior tests plus Ruff, ShellCheck, Python compile, and diff checks prove the issue-33 receipt contracts — `<local HK state not exported>`
- `mise run check`: fail (exit 101) — attempted to validate: Final canonical Foreman gate after entrypoint fixes; classify unchanged tmux E2E separately if it flakes — `<local HK state not exported>`
- `bash -lc '/usr/bin/python3 tests/agent_entrypoints_test.py && uvx ruff check tests/agent_entrypoints_test.py && git diff --check'`: pass (exit 0) — validates: Context engineering update routes agents and humans to the unattended entrypoint contract; focused suite and reference checks pass — `<local HK state not exported>`
- `bash -lc 'summary=/tmp/foreman-agent-real-e2e.tulhs7/artifacts/summary.json; jq -e '"'"'(.setup1Exit == 0) and (.setup2Exit == 0) and (.resumeExit == 0) and (.setup1.ready == true) and (.setup2.ready == true) and (.setup1.doctor.status == "passed") and (.setup2.doctor.status == "passed") and (.resume.doctor.status == "ok") and (.resume.optional_tools.hk.available == true) and (.resume.repo.dirty == false)'"'"' "$summary" >/dev/null && cmp /tmp/foreman-agent-real-e2e.tulhs7/artifacts/status.before /tmp/foreman-agent-real-e2e.tulhs7/artifacts/status.after-setup-1 && cmp /tmp/foreman-agent-real-e2e.tulhs7/artifacts/status.after-setup-1 /tmp/foreman-agent-real-e2e.tulhs7/artifacts/status.after-setup-2 && cmp /tmp/foreman-agent-real-e2e.tulhs7/artifacts/status.before-resume /tmp/foreman-agent-real-e2e.tulhs7/artifacts/status.after-resume'`: pass (exit 0) — validates: Disposable real-tool clone proves setup/resume against actual mise, foreman, Git, and HK without tracked drift — `<local HK state not exported>`
- `mise run verify-agent-entrypoints`: pass (exit 0) — validates: Encoded real-tool E2E builds Foreman and proves setup/resume in an isolated disposable clone with actual mise, Git, HK, and pinned CI contract — `<local HK state not exported>`
- `bash -lc 'ruby -e '"'"'require "yaml"; YAML.load_file(ARGV[0])'"'"' .github/workflows/ci.yml && uv run --python 3.12 python - <<'"'"'PY'"'"'
import tomllib
with open("Cargo.toml", "rb") as f: main = tomllib.load(f)
with open("Cargo.docker.toml", "rb") as f: docker = tomllib.load(f)
assert set(main["dependencies"]) == set(docker["dependencies"])
PY
'`: pass (exit 0) — validates: CI stabilization keeps Docker proof independent, syncs direct Docker dependencies, and preserves local heavy-gate behavior — `<local HK state not exported>`
- `mise run verify-release`: pass (exit 0) — validates: Decomposed PR Full Validation preserves unique proof while release gauntlet passes locally — `<local HK state not exported>`

## Readiness
- context: info — context recorded
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: pass — validation evidence with rationale recorded
- review: pass — external-enough review recorded
- profile-check:fast-gate: pass — validation dangerously skipped: fast-gate; reason: CI-only decomposition does not change runtime code; previous fast-gate attempts pass all changed tests and 375 Rust tests before the unchanged tmux timing flake.; mitigation: Final PR CI independently requires Quality Gate, Real Agent Entrypoints, Docker Build, and Full Validation.
- profile-check:heavy-gate: pass — validation dangerously skipped: heavy-gate; reason: Local Docker daemon is unavailable, so the complete local heavy gate cannot run here.; mitigation: Release gauntlet and real entrypoint E2E pass locally; final CI must pass Docker Build and decomposed Full Validation.
- profile-check:macos-overlay-required-lane: pass — validation dangerously skipped: macos-overlay-required-lane; reason: No overlay or Swift code changes exist.; mitigation: CI decomposition and entrypoint reviews passed; changed paths do not affect the overlay.
- profile-review:codex-review: pass — required profile review recorded: codex-review (matched .github/workflows/ci.yml, .mise/tasks/check, .mise/tasks/smoke-ghostty-display, +19 more)

## Review
- subagent / reviewer-fresh-context-gpt-5.6-sol [codex-review]: Issue #33 review requested changes: P1 portable command paths, P1 canonical log resolution, P1 typed/validated receipt schema, and P2 actionable diagnostics. Artifact: .pi-subagents/artifacts/ac0316ba-7c56-4709-8b3b-d67e774e94a7_reviewer_output.md paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +30 more. [changes-requested]
- subagent / reviewer-fresh-context-gpt-5.6-sol [architecture-polish-review]: B- architecture polish review; no P0, three P1 and one P2 must be fixed before acceptance. Artifact: .pi-subagents/artifacts/ac0316ba-7c56-4709-8b3b-d67e774e94a7_reviewer_output.md paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +30 more. [changes-requested]
- subagent / reviewer-fresh-context [codex-review]: Four bounded fresh-context passes reviewed the issue-33 diff. Terra-medium worker fixed the original portable command/log/schema/diagnostic findings; parent follow-ups fixed schema-invalid exit propagation, exact doctor finding/fix fields, empty override semantics, structured Git-state failures, wrong-typed provider crashes, and git-status failure handling. Final targeted review passed with no P0-P2 findings. paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +26 more. [accepted]
- subagent / reviewer-fresh-context [architecture-polish-review]: Architecture review confirms thin adapters preserve canonical mise/doctor/HK/log ownership, commands are argv+cwd structured, receipt schema is exact and typed, resume remains read-only, and nine isolated behavioral tests are wired into the canonical check. paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +26 more. [accepted]
- subagent / reviewer-fresh-context: Targeted context/docs review confirmed lean root agent guidance, human README discovery, docs indexes, and detailed workflow ownership. Replaced prose-fragile routing assertions with exact link/anchor checks; 10 focused tests pass. paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +30 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Targeted context/docs review confirmed agent/human routing and actual entrypoint semantics; exact link assertions replaced prose-fragile checks and 10 focused tests pass. paths: AGENTS.md, README.md, docs/AGENTS.md, +3 more. [accepted]
- subagent / reviewer-fresh-context: Fresh real-E2E review found HK status was too loosely asserted and CI used a mutable version tag. Fixed by requiring the exact newly created active_work suffix and pinning Harness Kit v0.3.0 to full commit c4bde2dbe1600a4aea7239ed40a500fb175ab182. Local real E2E passes after fixes. paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +33 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Real-E2E review findings fixed: HK receipt is bound to the newly started work item, Harness Kit CI install is pinned to immutable v0.3.0 commit c4bde2dbe1600a4aea7239ed40a500fb175ab182, docs match, and the local real-tool task passes. paths: .github/workflows/ci.yml, .mise/tasks/verify-agent-entrypoints, tests/agent_entrypoints_real_e2e.py, +1 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Targeted context/docs review replaced prose-fragile assertions with exact entrypoint and workflow-link checks; 10 focused tests pass and the new real-E2E task is documented across agent/human entry surfaces. paths: tests/agent_entrypoints_test.py. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Fresh CI review passed: local verify still builds Docker, PR Full Validation skips only its Docker subphase, dedicated cache-aware Docker Build preserves image proof, release compile concurrency is bounded, direct dependencies are synchronized, and docs match. paths: .github/workflows/ci.yml, .mise/tasks/verify, Cargo.docker.toml, +2 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Fresh review passed: local verify is restored unchanged; PR Quality Gate, Real Agent Entrypoints, Docker Build, and decomposed Full Validation preserve every unique proof without rerunning duplicated phases. paths: .github/workflows/ci.yml, docs/workflows.md. [accepted]

## Sync exclusions
- .pi-subagents: Runner-provided prompts, transcripts, and required child output are local orchestration artifacts, not repository source.
- .pi-subagents: Runner-provided prompts, transcripts, and required child output are local orchestration artifacts, not repository source.

## Dangerous skips
- validation: fast-gate — reason: mise run check passes formatting, Rust lint/typecheck, all nine new entrypoint tests, 375 Rust unit tests, and every preceding integration lane, then repeatedly flakes in unchanged tests/runtime_dashboard.rs::interactive_binary_popup_focus_action_exits_after_success; no Rust production or runtime-dashboard files are changed by origin/main...HEAD.; mitigation: Focused entrypoint tests, Ruff, Python compile, Bash syntax, ShellCheck, and diff checks pass; the full gate was attempted three times and CI must provide clean-runner evidence for the unrelated tmux timing lane.
- validation: heavy-gate — reason: The active HK item predates the rebase that brought merged Foreman #34 onto the branch, so profile matching reports Cargo paths that are upstream baseline and absent from git diff origin/main...HEAD.; mitigation: The reviewed branch diff contains only issue-33 entrypoints, check wiring, docs, focused tests, and HK export; no Cargo manifests or Rust source changed.
- validation: macos-overlay-required-lane — reason: The active HK item predates the rebase that brought merged Foreman #34 onto the branch, so profile matching reports control_api/overlay paths that are upstream baseline and absent from git diff origin/main...HEAD.; mitigation: No Swift, overlay, or Rust control API file changes exist in origin/main...HEAD; focused entrypoint proof and canonical check coverage were run.
- validation: fast-gate — reason: Only context/docs and the focused routing assertion changed after the previously attempted full gate. The full gate repeatedly passes format, Rust lint/typecheck, all entrypoint tests and 375 Rust tests before the unchanged runtime-dashboard tmux timing flake.; mitigation: Ten focused entrypoint/context tests, Ruff, reference checks, ShellCheck, and diff checks pass; PR Quality Gate is green and Full Validation remains the clean-runner broad evidence.
- validation: heavy-gate — reason: HK path matching still includes merged #34 Cargo baseline that is absent from origin/main...HEAD; the context update adds no Cargo or Rust changes.; mitigation: Branch diff and review are limited to entrypoints, docs/context, focused tests, check wiring, and HK export.
- validation: macos-overlay-required-lane — reason: HK path matching still includes merged #34 control-api baseline that is absent from origin/main...HEAD; the context update adds no Swift, overlay, or Rust changes.; mitigation: Targeted context/entrypoint tests and review pass; no overlay lane is affected.
- validation: fast-gate — reason: The new real-tool lane is intentionally separate from the fast gate and passed locally. The previously attempted fast gate repeatedly passes format, Rust lint/typecheck, all focused entrypoint tests, and 375 Rust tests before the unchanged runtime-dashboard tmux timing flake.; mitigation: Ten controlled entrypoint tests and the new real mise/Foreman/Git/HK disposable-clone E2E pass; PR CI now has a dedicated Real Agent Entrypoints job plus Quality Gate.
- validation: heavy-gate — reason: HK path matching includes merged #34 Cargo baseline and the workflow file; no Cargo or Rust source changes exist in origin/main...HEAD. Full Validation remains CI-owned and historically hits unrelated runner/tmux failures.; mitigation: Dedicated real entrypoint E2E and Quality Gate cover this change; Full Validation is still required as CI evidence.
- validation: macos-overlay-required-lane — reason: HK path matching includes merged #34 control-api baseline absent from origin/main...HEAD; no Swift, overlay, or control-api source changed.; mitigation: Real entrypoint E2E, focused tests, and CI Quality Gate cover the changed surfaces.
- validation: fast-gate — reason: The new CI-only split does not change runtime code; prior fast-gate attempts pass all changed entrypoint tests and 375 Rust tests before the unchanged tmux timing flake.; mitigation: Quality Gate and dedicated Real Agent Entrypoints are green; final CI rerun will separately prove Docker Build and Full Validation.
- validation: heavy-gate — reason: Local Docker daemon is unavailable, so the split Docker image lane cannot run locally; Full Validation previously loses runner communication during its inline Docker build.; mitigation: The final PR CI now runs Docker Build on a separate Buildx runner and Full Validation without inline Docker; both must pass before merge.
- validation: macos-overlay-required-lane — reason: No macOS overlay or Swift code changes exist in the CI stabilization diff.; mitigation: Changed surfaces are CI workflow, Docker build metadata, verify orchestration, and docs only.
- validation: fast-gate — reason: CI-only decomposition does not change runtime code; previous fast-gate attempts reached only the known unchanged tmux timing flake after all changed tests and 375 Rust tests passed.; mitigation: Final PR CI independently requires Quality Gate, Real Agent Entrypoints, Docker Build, and Full Validation.
- validation: heavy-gate — reason: Local Docker daemon is unavailable; the complete local heavy gate cannot run here.; mitigation: Release gauntlet passes locally; final CI must pass separate Docker Build and decomposed Full Validation before merge.
- validation: macos-overlay-required-lane — reason: No overlay or Swift code changes exist.; mitigation: CI/workflow decomposition review passed and all changed entrypoint tests remain green.
- validation: fast-gate — reason: CI-only decomposition does not change runtime code; previous fast-gate attempts pass all changed tests and 375 Rust tests before the unchanged tmux timing flake.; mitigation: Final PR CI independently requires Quality Gate, Real Agent Entrypoints, Docker Build, and Full Validation.
- validation: heavy-gate — reason: Local Docker daemon is unavailable, so the complete local heavy gate cannot run here.; mitigation: Release gauntlet and real entrypoint E2E pass locally; final CI must pass Docker Build and decomposed Full Validation.
- validation: macos-overlay-required-lane — reason: No overlay or Swift code changes exist.; mitigation: CI decomposition and entrypoint reviews passed; changed paths do not affect the overlay.
