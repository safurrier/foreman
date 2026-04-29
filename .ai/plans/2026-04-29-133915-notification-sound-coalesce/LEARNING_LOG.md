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

## 2026-04-29 14:18 — Review follow-up and retrospective

Codex review correctly pointed out that mixed completion plus attention bursts
needed an explicit sound priority. Updated the reducer so `needs attention`
wins the single audible slot, then added a regression test for that exact
sequence. The sequence matters because completion notifications are debounced
until the second idle observation while attention notifications fire
immediately.

Also updated root `SPEC.md` because grouped notifications and audio priority
are durable user-visible behavior, not just plan-local notes.

**What matched the plan:** Coalescing fit cleanly in the reducer and sound
gating fit cleanly in the dispatcher request model.

**What diverged:** Mixed-kind bursts needed explicit priority, and the root spec
needed an update. Both were review findings rather than initial scope bullets.

**One-shot next time:** Include mixed-kind notification cases and canonical spec
updates in the first implementation plan for user-visible behavior changes.
