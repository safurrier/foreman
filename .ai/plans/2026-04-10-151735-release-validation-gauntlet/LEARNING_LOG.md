---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

- 2026-04-10 15:17 MST — Planning started from operator feedback that the app
  can still feel inconsistent in real use even when targeted tests are green.
  The main conclusion is that Foreman now needs a checklist-driven release
  gauntlet, not just more isolated smoke tests.
- 2026-04-10 15:17 MST — Current coverage is broad but fragmented. `runtime_*`,
  `tmux_*`, `notification_runtime`, and native hook tests prove many seams, but
  the proof is spread across files and not yet organized into a single
  "everything important actually works together" validation story.
- 2026-04-10 16:11 MST — The first compiled-binary gauntlet pass exposed two
  real mismatches between assumptions and product behavior. Harness-view cycling
  includes empty Gemini and OpenCode filters before returning to `all`, and PR
  lookup failures updated UI state without always producing an explicit
  operator-alert log entry. The gauntlet was updated for the full harness cycle,
  and runtime now logs pull-request alert transitions when they become active.
- 2026-04-10 16:24 MST — The release lane initially failed the fast gate
  because shared test-support modules are compiled into multiple test targets,
  so target-specific helpers look dead from other test crates. The correct fix
  was to mark the support modules as allowed dead code at the module boundary,
  not to weaken repo-wide linting.
- 2026-04-10 16:36 MST — Notification timing needed state-based synchronization.
  Sleep-only assertions were flaky under the full heavy suite because the
  intermediate `working` observation could be missed between two
  `needs_attention` transitions. Waiting for the actionable preview row to show
  `working` and `needs attention` made the gauntlet deterministic without
  changing product semantics.
- 2026-04-10 16:47 MST — The slice completed with a dedicated
  `mise run verify-release` entrypoint, a generated report artifact, and `mise
  run verify` updated to include the release-confidence gauntlet before the UX
  capture phase.
