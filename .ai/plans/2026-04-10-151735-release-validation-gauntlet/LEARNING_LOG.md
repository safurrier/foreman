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
