# HK export: `2026-05-10-155836-foreman-hotkey-hk-migration`

This directory is a generated review/handoff package from the Harness Kit ledger. Do not hand-edit it; update HK with `hk plan`, `hk decide`, `hk validate`, `hk review add`, and `hk sync`, then regenerate.

## Freshness
Validate this export against local HK state with:

```bash
hk export --format handoff-dir --output .ai/hk/2026-05-10-155836-foreman-hotkey-hk-migration --target . --check
```

Historical hand-authored slice plans live under `.ai/plans/`; new Harness Toolkit repo work should use HK and generated `.ai/hk/` exports.

## Handoff

## Summary
- Work: `2026-05-10-155836-foreman-hotkey-hk-migration`
- Branch: `fix/macos-hotkey-recorder-default`
- Git SHA: `bcc0bd0`
- Dirty: `true`
- Sync status: `synced`

## Context
- None recorded.

## Plan
- Fix macOS overlay shortcut recorder/default issues, harden GUI-launched tmux inventory, and migrate new Foreman work guidance from legacy .ai/plans to Harness Kit exports.

## Decisions and spec reflection
- Use KeyboardShortcuts for both persisted macOS shortcut recording and normal global handling; keep custom Carbon registration only for FOREMAN_OVERLAY_HOTKEY development overrides.
- Migrate new Foreman lifecycle guidance from legacy .ai/plans to Harness Kit, with compact .ai/hk/<work-id> exports as generated handoff packages.
- Use Cmd+Option+F as the default macOS shortcut because Cmd+Shift+F is reported by macOS as an enabled system symbolic hotkey.
  - Spec: updated: Spec/docs updated or verified.; refs: docs/macos-overlay/architecture.md, docs/macos-overlay/validation.md
  - Spec: updated: Spec/docs updated or verified.; refs: AGENTS.md, docs/workflows.md, docs/tour.md
  - Spec: updated: Spec/docs updated or verified.; refs: docs/macos-overlay/validation.md

## Learning
- None recorded.

## Gaps
- None recorded.

## Validation evidence
- `cargo test -q adapters::tmux::tests::parse_pane_records_skips_malformed_lines`: pass (exit 0) — validates: Regression for malformed tmux pane records so one bad line does not collapse macOS inventory. — `.harness-local/harness-kit/root/work/2026-05-10-155836-foreman-hotkey-hk-migration/artifacts/ev_20260510_155933_567117.transcript.log`
- `mise run validate-macos-overlay-change`: pass (exit 0) — validates: Required lane for macOS overlay shortcut, Settings, snapshot, and app-bundle behavior. — `.harness-local/harness-kit/root/work/2026-05-10-155836-foreman-hotkey-hk-migration/artifacts/ev_20260510_155943_370186.transcript.log`
- `mise run check`: pass (exit 0) — validates: Fast quality gate passes after macOS shortcut, tmux locale, and HK migration changes. — `.harness-local/harness-kit/root/work/2026-05-10-155836-foreman-hotkey-hk-migration/artifacts/ev_20260510_160204_295924.transcript.log`
- `sh -c 'hk sync --target . --json && hk ready --target . --json || true'`: pass (exit 0) — validates: HK lifecycle status was inspected after sync, validation, and review records; readiness output is captured for handoff. — `.harness-local/harness-kit/root/work/2026-05-10-155836-foreman-hotkey-hk-migration/artifacts/ev_20260510_161455_245821.transcript.log`

## Readiness
- Status: `ready`
- context: info — no context recorded; okay for trivial work, add hk context if it prevents rediscovery
- plan: pass — plan recorded
- decision: pass — decision and spec reflection recorded
- validation: pass — validation evidence with rationale recorded
- review: pass — external-enough review recorded
- profile-check:fast-gate: pass — required profile check recorded: fast-gate (matched .ai/hk/AGENTS.md, .ai/plans/2026-05-10-144612-macos-hotkey-recorder-default/IMPLEMENTATION.md, .ai/plans/2026-05-10-144612-macos-hotkey-recorder-default/LEARNING_LOG.md, +36 more)
- profile-check:macos-overlay-required-lane: pass — required profile check recorded: macos-overlay-required-lane (matched apps/macos-overlay/Sources/ForemanOverlay/GauntletHooks.swift, apps/macos-overlay/Sources/ForemanOverlay/HotkeyController.swift, apps/macos-overlay/Sources/ForemanOverlay/SettingsPanelController.swift, +5 more)
- profile-check:handoff-readiness: pass — required profile check recorded: handoff-readiness (matched .ai/hk/AGENTS.md, .ai/plans/2026-05-10-144612-macos-hotkey-recorder-default/IMPLEMENTATION.md, .ai/plans/2026-05-10-144612-macos-hotkey-recorder-default/LEARNING_LOG.md, +36 more)
- profile-review:codex-review: pass — required profile review recorded: codex-review (matched .mise/tasks/plan, .mise/tasks/pr-preflight, src/adapters/tmux.rs)
- sync: pass — sync checkpoint fresh

## Review
- subagent / reviewer-fresh-context [codex-review] (code-diff, macos-shortcut, hk-migration): Fresh-context reviewer found two blockers: KeyboardShortcuts.isEnabled was not OS-level registration proof, and tmux parse-error skipping was too broad. Both were fixed: Settings now says configured rather than Carbon-registered for recorder shortcuts, and tmux parsing skips only non-tabbed noise while erroring on all-malformed/truncated tabbed records. [accepted]

## Sync exclusions
- .pi, HK_PROVIDER_EXPLORATION.md: agent-local pi state and unrelated provider exploration note are intentionally not part of this branch
- .pi, HK_PROVIDER_EXPLORATION.md: agent-local pi state and unrelated provider exploration note are intentionally not part of this branch
- .pi, HK_PROVIDER_EXPLORATION.md: agent-local pi state and unrelated provider exploration note are intentionally not part of this branch
- .pi, HK_PROVIDER_EXPLORATION.md: agent-local pi state and unrelated provider exploration note are intentionally not part of this branch
- .pi, HK_PROVIDER_EXPLORATION.md: agent-local pi state and unrelated provider exploration note are intentionally not part of this branch
- .pi, HK_PROVIDER_EXPLORATION.md: agent-local pi state and unrelated provider exploration note are intentionally not part of this branch
- .pi, HK_PROVIDER_EXPLORATION.md: agent-local pi state and unrelated provider exploration note are intentionally not part of this branch
- .pi, HK_PROVIDER_EXPLORATION.md: agent-local pi state and unrelated provider exploration note are intentionally not part of this branch
