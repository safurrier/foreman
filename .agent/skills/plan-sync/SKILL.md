---
name: plan-sync
description: Audit a named legacy .ai/plans artifact without making it current policy. Use only when maintaining historical pre-Harness Kit plan evidence; use HK for all new work.
allowed-tools: Read, Glob, Grep, Bash
---

# Legacy plan audit

Never create or select an "active" `.ai/plans/**` directory. Harness Kit owns current planning, validation, sync, readiness, and handoff evidence.

Use this compatibility skill only when a task explicitly maintains one named historical plan:

1. Read `.ai/plans/AGENTS.md` and the named plan.
2. Verify internal links and required historical files. Avoid comparing its status to the current branch or PR.
3. Preserve original dates, decisions, and validation claims.
4. Report contradictions with current policy as historical context. Don't rewrite them into current instructions.
5. Route reusable lessons to the current `AGENTS.md` or `docs/` owner.

For current work, stop and use `hk status`, `hk validate`, `hk sync`, and `hk ready` instead.
