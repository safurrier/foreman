---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-19 17:03 MST

- The current build already has one compound sort mode: `attention` sorts by status rank and then recent activity. The real gap is that `recent` is not presented as an equally explicit multi-axis preset, so the UI does not communicate the model clearly.
- Status symbols already exist in the sidebar and detail surfaces. The request is mainly a semantic color pass plus, if needed, a small glyph audit rather than a new icon system.
- The current product language still says `View: recent` and `View: attention`, not a literal `Inbox` tab, so this slice should talk about the operator list and attention view instead of inventing a second information architecture change.

## 2026-04-19 18:54 MST

- The first full-gate failure after implementation was not sort logic. It was stale compiled-binary wording in `tests/release_gauntlet.rs` still expecting the old single-axis `attention` label. Tightening that test to `View: attention->recent` turned the release gauntlet into a real proof of the new contract instead of a loose side effect.
- The live runtime smoke had the same issue in softer form. It was still matching `View: attention`, which would have kept passing even if the explicit dual-sort label regressed. Making it assert `View: attention->recent` closed that gap.
- Theme-relative status color retuning was the right scope. The code already had semantic theme tokens, so the slice only needed palette tuning for `working`, plus a terminal-theme assertion, not a new global color system.
- The user explicitly wanted the full validation ladder for the slice, so the plan needed to name `mise run verify` and strict `verify-native` up front. That prevented the common failure mode where a UI/state slice stops after unit tests even though compiled-binary and real-provider lanes are part of the product contract here.
