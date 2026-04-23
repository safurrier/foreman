---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-18 14:07 MST

- Started this slice because `verify-native` proves provider integration in temp harnesses, but it does not fully prove the installed dashboard/UI behaviors the user actually cares about.
- Existing release/runtime gauntlets already prove `f` under compatibility-mode paths, so this slice should add deterministic native-hook dashboard proof rather than invent a brand-new harness.
- `mise run plan` could not scaffold automatically because the repo's branch detection is still confused by stale git metadata, so the plan directory was created manually using the repo template contract.

## 2026-04-18 15:36 MST

- The first pass of the new runtime smokes failed because the tests were reasoning from row order instead of the real dashboard selection model. Searching for the target session before asserting details made the checks stable and closer to operator behavior.
- `f` needed a stronger live proof than reducer coverage. Updating `focus_pane` to switch the tmux client session before selecting the window and pane closed the cross-session gap the user saw locally.
- `mise run check` looked hung during `cargo test --all-features`, but the issue was buffered output, not a deadlock. Checking the active test binary before interrupting the run is the right move for this repo.

## Retrospective

- Matched the plan: the slice stayed narrow, used the existing tmux fixtures, and landed deterministic native-hook dashboard coverage plus a live cross-session focus proof.
- Diverged from the first implementation sketch: the tests needed search-driven selection instead of raw `j/k` counts because the sidebar rows are not a stable contract across views.
- One-shot next time would be easier if the runtime smoke helpers exposed a tiny `select_search_match()` utility. The missing helper is why the first version duplicated brittle key sequences.
- Most useful skills: `testing-core`, `development-debugging`, and `interacting-with-tmux`.
- Least useful baseline skills for this Rust slice: the Python-specific ones were loaded per repo policy but did not materially shape the implementation.
