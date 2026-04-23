# Native Dashboard Smoke

## Problem

`verify-native` proves that Foreman's real Claude/Codex/Pi integrations still work in controlled temp environments, but it does not fully prove the interactive dashboard behaviors a local operator cares about:

- native status provenance surfacing in the dashboard
- native attention ordering in the attention-first view
- tmux focus via `f` in the live dashboard

That leaves a gap between provider-contract confidence and operator-surface confidence.

## Scope

Add deterministic tmux-backed smoke coverage for:

- Claude native hook status surfacing as `Status source: native hook`
- Claude native `needs_attention` behavior surfacing in `View: attention`
- dashboard `f` key focusing the intended pane from a live dashboard path

## Non-Goals

- Replace real-provider `verify-native`
- Add new provider-specific native attention semantics for Codex
- Rework the dashboard information architecture

## Acceptance Criteria

- A deterministic tmux-backed test starts a live Foreman dashboard, injects Claude native hook events, and observes `Status source: native hook`.
- The same or adjacent test proves that a Claude `needs_attention` native signal is surfaced while the dashboard is in `View: attention`.
- A tmux-backed smoke path proves `f` focuses the intended pane from the live dashboard and not only via reducer/unit coverage.
- The new coverage runs in the normal repo test lanes without requiring real provider auth.
- `mise run check` passes.
- `mise run verify` passes.

## Constraints

- Use the existing tmux test harnesses instead of adding a second bespoke harness.
- Prefer deterministic hook-binary invocation over provider-network behavior.
- Keep the slice narrow and test-behavior oriented.
