---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

## 2026-05-02

- PASS: docs workflow verifier against the repo root
- PASS: `vhs demos/readme-quickstart.tape`
- PASS: extracted and inspected a frame from `demos/readme-quickstart.gif`; the demo uses an isolated tmux socket and `/tmp/foreman-readme-demo/...` paths.
- PASS: extracted and inspected search and flash-jump frames from `demos/readme-quickstart.gif`.
- PASS: refreshed demo with Foreman `terminal` theme under VHS `Cobalt Neon` and inspected rendered search/flash frames for color.
- PASS: `mise run check`
