---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — sidebar-density-pass

## Problem

Foreman is materially better than the initial shell, but the live walkthrough still
shows three concrete UX failures:

1. navigation and focus feel inconsistent because some rows are selectable but not
   reliably actionable
2. the UI is still too text-heavy at rest, especially in the sidebar, footer, and
   help surface
3. performance regressions are discoverable only by feel because the runtime does not
   emit timing telemetry and the heavy gate does not assert against interaction lag

This slice tightens the browsing/action model, reduces idle information density, and
adds enough runtime profiling to catch future regressions during heavy validation.

## Requirements

### MUST

- `j`/`k` movement must no longer synchronously trigger pull-request shell-outs on
  every selection change.
- The runtime must emit timing telemetry into the existing run log for at least:
  action handling, inventory refresh, and pull-request lookup.
- Slow paths must be surfaced explicitly in the run log so operator reports can be
  tied back to concrete timings.
- Heavy validation must include a scripted smoke that proves navigation no longer
  performs pull-request lookups for every intermediate row.
- Session and window rows must support a consistent “actionable pane” resolution so
  focus and compose commands do not silently no-op when the selection is not already
  on a pane.
- Help must include a legend for harness badges and status indicators.
- The default interaction for composing input must become `Enter` to send, with a
  documented newline alternative and backward-compatible fallback.
- The resting footer must be quieter than the current two-line wall of shortcuts.
- Non-agent panes must remain filtered out by default.

### SHOULD

- Sidebar rows should be denser and more scannable without depending on color alone.
- The preview and compose panes should explain the current target in fewer words.
- The validation trail should capture the live UX/perf smoke alongside normal
  `check`/`verify` results.

## Constraints

- Stay within the existing action/reducer/effects architecture.
- Prefer logging-based profiling over introducing a new tracing dependency.
- Do not make normal `cargo test` materially slower; heavier interactive perf checks
  belong in the UX/heavy gate.
- Keep glyph-based affordances compatible with the existing `no-color` fallback.
