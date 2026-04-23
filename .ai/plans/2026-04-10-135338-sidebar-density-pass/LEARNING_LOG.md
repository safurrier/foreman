---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-10

- Live repro confirmed that the lag complaint was grounded in architecture, not
  perception: the runtime called selected pull-request refresh immediately after
  key actions, and that path can shell out through `git` and `gh`.
- Focus inconsistency also had a real source: session/window rows were selectable,
  but `Enter` only collapsed sessions or focused panes. Window rows were effectively
  dead ends for primary action.
- The existing run log already gives a low-friction place for profiling data, so
  adding timing telemetry there is cheaper and more operator-friendly than bringing
  in a separate tracing/export stack for v1.
- The heavy perf smoke works best as an ignored integration test invoked from the
  UX/heavy lane, not the default `cargo test` path. It is valuable, but too slow
  to tax every fast test run.
- A fake `gh` subprocess log is not enough to prove the runtime finished a lookup;
  the smoke also has to wait for the run log timing entry because the fake backend
  can append its marker before Foreman writes the timing line.
