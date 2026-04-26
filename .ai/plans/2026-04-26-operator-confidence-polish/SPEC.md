# SPEC — Operator confidence polish

## Problem

Foreman is mechanically capable, but the operator still needs more confidence when the dashboard is empty, degraded, or using compatibility fallbacks. Current Claude Code panes can also appear as `ERROR` because compatibility parsing treats stale scrollback hook/tool failures as current status.

## Requirements

- Fix Claude Code compatibility status so stale errors in older scrollback do not override current prompt/ready lines.
- Keep real recent error text classified as error.
- Add actionable empty/loading/degraded guidance in the main pane.
- Add a resolved config/readout utility covering config path, UI state, cache path, and key runtime values.
- Improve doctor next steps with copy-paste setup/config/cache commands where safe.
- Add a lightweight in-app recent event/inbox surface using existing notification/status data.
- Add a minimal activity digest for visible agents.
- Add first-run/setup onboarding copy in the main pane when relevant diagnostics are present.
- Add Esc as a normal-mode quit equivalent without breaking modal/input/help behavior.

## Non-Goals

- Full fuzzy command palette implementation.
- Persistent user pin/favorites model.
- Live GitHub CI/check-run integration.
- A new workspace-first navigation tree.

## Acceptance

- Focused tests cover Claude stale-error parsing, recent-error parsing, config readout, and Esc behavior.
- Existing runtime dashboard tests pass.
- `mise run check` passes.
