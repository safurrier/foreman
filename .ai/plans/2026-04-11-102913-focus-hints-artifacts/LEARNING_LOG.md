---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

- The current UI already contains most of the state needed for this slice.
  What is missing is better presentation logic and proof surfacing, not a new
  runtime subsystem.
- The first runtime and gauntlet failures were proof drift, not product bugs.
  The help surface got taller after adding provenance guidance, so the old
  scroll counts and section assertions were no longer honest.
- Focus-aware help is only useful if the tests assert the real focus at the
  moment help opens. In the broader release walkthrough, help correctly opened
  from `Compose`, not `Sidebar`, because the operator had already used the
  compose path.
- CI artifact surfacing did not need a new generation pipeline. Reusing the
  existing UX and release artifact directories kept the workflow simple and
  aligned with the heavy local validation loop.
