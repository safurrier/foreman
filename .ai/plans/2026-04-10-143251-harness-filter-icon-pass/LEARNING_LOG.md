---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-10

- The “tiny logos” request is better interpreted as “more visual harness
  identity,” not literal brand image rendering. Ratatui + Crossterm + tmux is a
  text-cell environment, so portable marks with fallbacks are the right target.
- The next real UX gain is not another palette tweak. It is reducing sidebar
  decoding cost and making act-on-selection behavior obvious from the current
  selection.
- The medium-width header had drifted back into information overload; trimming
  it was necessary not just for aesthetics but to keep operationally important
  labels like `notify=MUTED` from being clipped out of the buffer tests.
- A full-suite tmux inventory test caught a real behavioral regression: when
  pane-level shell visibility was enabled, non-agent-only sessions started
  leaking back into view. Session visibility has to key off matching agent panes
  first, then treat session-level shell visibility as a separate explicit toggle.
- The most stable live tmux UX smoke is sequence-based, not copy-fragile. The
  harness-filter walkthrough worked once the test asserted on state progression
  and final action delivery instead of a single exact header string.
