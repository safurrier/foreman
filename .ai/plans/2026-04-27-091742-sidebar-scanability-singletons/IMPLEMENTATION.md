---
id: plan-implementation
title: Implementation Plan
description: Step-by-step approach for this unit of work.
---

# Implementation — sidebar-scanability-singletons

1. Trace visible target construction, sidebar rendering, and reducer selection behavior.
2. Add a display-level decision to elide singleton window rows in agent-first views.
3. Preserve topology by keeping window rows when sessions/windows are explicitly shown or when windows have multiple visible panes.
4. Adjust counts/labels so collapsed singletons do not repeat `1w/1p` and `1p` noise.
5. Update render/runtime/release tests for new sidebar shape.
6. Run focused tests, `mise run check`, `mise run verify-ux`, then push and poll PR CI/review.
