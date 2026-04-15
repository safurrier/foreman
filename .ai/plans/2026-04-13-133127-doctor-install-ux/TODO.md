---
id: plan-todo
title: Task List
description: >
  Checkable tasks for this unit of work. Check off as you go.
  See _example/ for a reference.
---

# TODO — doctor-install-ux

- [x] Confirm the local failure mode with real machine evidence: installed binaries, missing config, empty native dirs, and fallback-heavy logs
- [x] Inspect repo-local and global hook wiring for Claude, Codex, and Pi in affected repos
- [x] Inspect the current install, config, bootstrap, and native-preflight surfaces
- [x] Write a scoped spec for `doctor` and install/diagnose UX
- [x] Break the work into phases with clear ownership boundaries and validation
- [x] Decide the initial CLI surface: flat utility flags vs subcommand
- [x] Define the doctor findings model and JSON contract
- [x] Define safe auto-fix boundaries by provider, especially Claude config merge behavior
- [x] Define runtime and TUI diagnostic surfaces so compatibility fallback explains itself
- [x] Implement the slice in phased code changes with validation after each phase
