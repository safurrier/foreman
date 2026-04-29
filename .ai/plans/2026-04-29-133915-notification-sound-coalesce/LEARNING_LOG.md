---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-29 13:39 — Scoping

User feedback identified overlapping sounds as the worst part of notification
bursts, with stacked visible notifications as a related annoyance. The narrow
slice is to coalesce same-refresh notifications and add an audible gate instead
of redesigning the notification profile system.

Decision: do not add config in the first pass. If the hardcoded quiet default
is wrong, a later slice can expose thresholds or toggles after real use.

## 2026-04-29 13:55 — Subtitle assertion correction

The first focused run failed because the new tests expected pane titles in the
notification body. Existing notification copy prefers workspace names via
`Pane::navigation_title`, so `/tmp/alpha` becomes `alpha`. Updated the tests to
assert the actual operator-facing labels and to keep checking that full
workspace paths are not exposed.

## 2026-04-29 14:03 — Self-review

Ran the anti-pattern checklist against the diff. No blocking issues found.
The new string assertions are intentional because notification copy is the
observable behavior for this slice. The implementation keeps the existing
policy/reducer/dispatcher split and does not add compatibility shims or hidden
fallbacks.
