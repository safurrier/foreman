---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary for the operator-cockpit reframe.
---

# Learning Log

- 2026-04-11 16:18 MST — Product direction review showed that the current repo
  is stronger technically than it is as a product surface. Native-vs-compat
  seams, tests, and tmux control basics are in good shape. The weak spot is the
  default operator experience.
- 2026-04-11 16:18 MST — The right move is not to throw away the current spec.
  The better move is to keep the current integration and validation backbone,
  then rewrite the product contract around an attention-first cockpit.
- 2026-04-11 16:18 MST — The current tree and detail panes both carry some
  value, but neither is strong enough to own half the screen. The roadmap
  should merge them into a compact context surface and keep exact topology as a
  secondary view.
- 2026-04-11 16:18 MST — The four review findings are worth landing early.
  They are small compared to the reframe, but they directly affect operator
  trust and release proof.
- 2026-04-11 16:18 MST — Validation needs to be designed up front for the
  larger slices. Stable artifact paths, macOS PR proof, and early perf gates
  should be treated as part of the design, not follow-up cleanup.
