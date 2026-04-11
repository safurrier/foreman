---
id: plan-learning-log
title: Learning Log
description: >
  Notes captured during implementation for future reuse.
---

# Learning Log — help-scroll-footer-pass

- The first render-only change was straightforward; the real work was in the
  interaction contract. Help scrolling needed reducer-owned state so `j/k`,
  arrows, and page keys could become modal behavior without leaking widget-local
  mutation into the renderer.
- The release gauntlet exposed two brittle proof patterns:
  - waiting for generic words like `Search` is unsafe when those words also
    appear in recent output or other panels
  - sending a burst of keys immediately after a resize is flaky in the noisier
    multi-session walkthrough, even when the same burst passes in a smaller
    focused runtime smoke
- The right fix for those failures was not relaxing assertions. It was making
  the proof more specific:
  - assert overlay-unique text like `Query: ...`
  - pace scroll keys in the release gauntlet after the constrained-layout resize
  - wait for the dashboard to restabilize after closing help and resizing back
- `notification_runtime` was still timing-sensitive under the full
  `cargo test --all-features` sweep. The durable fix was to give the simulated
  `working -> idle` transition enough dwell time for the runtime to actually
  observe `working` before asserting the completion notification.
