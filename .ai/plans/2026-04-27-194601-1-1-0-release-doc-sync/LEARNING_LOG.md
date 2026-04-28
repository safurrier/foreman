---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

- 2026-04-27 19:46 MST — `Cargo.toml` and `README.md` already identify
  `1.1.0`; local and remote tags only include `v1.0.0`. The main release gap is
  the missing `CHANGELOG.md` entry. A SPEC line still described native
  integrations as a future rebuild order even though Claude, Codex, and Pi
  native support have shipped.
- 2026-04-27 19:51 MST — The docs verifier needs `python3` in this environment;
  `python` fails on the script's modern type syntax. `mise run check` and
  `mise run verify-release` both passed after the doc updates.
- 2026-04-27 19:57 MST — Opened PR #5 for the release-doc sync branch.
