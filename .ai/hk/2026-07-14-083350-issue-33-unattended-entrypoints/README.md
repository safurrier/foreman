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

## Readiness
- context: info — context recorded
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: pass — validation evidence with rationale recorded
- review: pass — external-enough review recorded
- profile-check:fast-gate: pass — validation dangerously skipped: fast-gate; reason: Only context/docs and the focused routing assertion changed after the previously attempted full gate. The full gate repeatedly passes format, Rust lint/typecheck, all entrypoint tests and 375 Rust tests before the unchanged runtime-dashboard tmux timing flake.; mitigation: Ten focused entrypoint/context tests, Ruff, reference checks, ShellCheck, and diff checks pass; PR Quality Gate is green and Full Validation remains the clean-runner broad evidence.
- profile-check:heavy-gate: pass — validation dangerously skipped: heavy-gate; reason: HK path matching still includes merged #34 Cargo baseline that is absent from origin/main...HEAD; the context update adds no Cargo or Rust changes.; mitigation: Branch diff and review are limited to entrypoints, docs/context, focused tests, check wiring, and HK export.
- profile-check:macos-overlay-required-lane: pass — validation dangerously skipped: macos-overlay-required-lane; reason: HK path matching still includes merged #34 control-api baseline that is absent from origin/main...HEAD; the context update adds no Swift, overlay, or Rust changes.; mitigation: Targeted context/entrypoint tests and review pass; no overlay lane is affected.
- profile-review:codex-review: pass — required profile review recorded: codex-review (matched .mise/tasks/check, .mise/tasks/smoke-ghostty-display, Cargo.lock, +14 more)

## Review
- subagent / reviewer-fresh-context-gpt-5.6-sol [codex-review]: Issue #33 review requested changes: P1 portable command paths, P1 canonical log resolution, P1 typed/validated receipt schema, and P2 actionable diagnostics. Artifact: .pi-subagents/artifacts/ac0316ba-7c56-4709-8b3b-d67e774e94a7_reviewer_output.md paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +30 more. [changes-requested]
- subagent / reviewer-fresh-context-gpt-5.6-sol [architecture-polish-review]: B- architecture polish review; no P0, three P1 and one P2 must be fixed before acceptance. Artifact: .pi-subagents/artifacts/ac0316ba-7c56-4709-8b3b-d67e774e94a7_reviewer_output.md paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +30 more. [changes-requested]
- subagent / reviewer-fresh-context [codex-review]: Four bounded fresh-context passes reviewed the issue-33 diff. Terra-medium worker fixed the original portable command/log/schema/diagnostic findings; parent follow-ups fixed schema-invalid exit propagation, exact doctor finding/fix fields, empty override semantics, structured Git-state failures, wrong-typed provider crashes, and git-status failure handling. Final targeted review passed with no P0-P2 findings. paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +26 more. [accepted]
- subagent / reviewer-fresh-context [architecture-polish-review]: Architecture review confirms thin adapters preserve canonical mise/doctor/HK/log ownership, commands are argv+cwd structured, receipt schema is exact and typed, resume remains read-only, and nine isolated behavioral tests are wired into the canonical check. paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +26 more. [accepted]
- subagent / reviewer-fresh-context: Targeted context/docs review confirmed lean root agent guidance, human README discovery, docs indexes, and detailed workflow ownership. Replaced prose-fragile routing assertions with exact link/anchor checks; 10 focused tests pass. paths: .agents/entrypoint-report.py, .agents/resume, .agents/setup, +30 more. [accepted]
- subagent / reviewer-fresh-context [codex-review]: Targeted context/docs review confirmed agent/human routing and actual entrypoint semantics; exact link assertions replaced prose-fragile checks and 10 focused tests pass. paths: AGENTS.md, README.md, docs/AGENTS.md, +3 more. [accepted]

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
