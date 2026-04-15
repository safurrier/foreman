---
id: plan-todo
title: Task List
description: >
  Checkable tasks for this unit of work. Check off as you go.
  See _example/ for a reference.
---

# TODO — native-e2e-hardening

- [x] Add a native preflight task that checks tmux, provider CLIs, Codex hook support, and common auth status seams
- [x] Tighten `verify-native` so strict closeout mode requires enabled real E2Es and runs preflight first
- [x] Add an explicit Codex hook/version floor to the real Codex E2E so old PATH installs fail clearly
- [x] Stabilize the Claude real E2E so completion notification proof is reliable on a correctly configured machine
- [x] Update README, workflow guidance, and agent guidance for strict native closeout and preflight usage
- [x] Run the full native lane plus the normal fast gate and record the results
