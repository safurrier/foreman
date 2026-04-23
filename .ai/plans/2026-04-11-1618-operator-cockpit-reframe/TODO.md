---
id: plan-todo
title: Task List
description: >
  Checkable tasks for the operator-cockpit reframe roadmap.
---

# TODO — operator-cockpit-reframe

- [x] Record the roadmap scope in a new plan slice.
- [ ] Rewrite top-level `SPEC.md` around the operator-cockpit product shape.
- [ ] Add ADR `0002` for information architecture and view model.
- [ ] Add ADR `0003` for lifecycle truth hierarchy and task identity.
- [ ] Add ADR `0004` for spawn transactions and repo/worktree model.
- [ ] Fix notification suppression to use actionable pane identity.
- [ ] Make per-pane tmux capture failure observable without breaking soft-fail behavior.
- [ ] Replace dated CI artifact upload paths with stable paths or a manifest.
- [ ] Add a macOS PR lane with at least build plus one targeted smoke path.
- [ ] Add profiling or guardrails that keep slow external lookups off the hot path.
- [ ] Add `Inbox`, `All Agents`, and `Topology` as explicit views.
- [ ] Replace the current detail panel with a combined context surface.
- [ ] Demote compose to a drawer or modal path.
- [ ] Add finer internal lifecycle and attention reasons while keeping the coarse public status model.
- [ ] Introduce repo/worktree records and cached branch or dirty state.
- [ ] Replace raw window spawning with a transaction-backed spawn flow and rollback journal.
- [ ] Introduce task identity separate from pane identity.
- [ ] Add hybrid snapshot-plus-event lifecycle reconciliation.
- [ ] Extend render, runtime, gauntlet, and performance proof for each phase.
- [ ] Sync spec, docs, ADRs, and plan logs after each landed slice.
