---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-30

- Loaded the foreman baseline skills and the external context-engineering
  README/skills.
- Context-engineering reported depth 0 for root `AGENTS.md`, contributor tier
  `2-5`, and no relevant prior Claude sessions.
- Reference validation found backticked prose that looked like broken paths:
  `artifacts/`, `_templates/`, `_example/`, and `j/k`.
- Antipattern validation found missing generated-by watermarks and that root
  `AGENTS.md` was 96 lines, above the depth-0 target of 80 lines.
- Rewrote root `AGENTS.md` to 69 lines with the lean context-engineering shape:
  self-documentation requirement, orientation, compact workflow, validated
  commands, gotchas, and related context.
- Kept the Commands section despite a depth-0 warning. In this repo the root is
  the single application boundary, and these commands are cross-cutting gates
  rather than module-specific choices.
