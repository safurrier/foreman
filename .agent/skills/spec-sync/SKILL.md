---
name: spec-sync
description: Route a branch diff through the repository's current SPEC and ADR contract owners. Use when reviewing whether changed behavior requires a contract update before handoff.
allowed-tools: Read, Edit, Write, Glob, Grep, Bash
---

# Contract sync compatibility route

Use the installed `context-contracts` workflow as the authority for `SPEC.md` and `docs/decisions/`. This repo-local skill only supplies Foreman routing context.

## Workflow

1. Inspect `git diff origin/main...HEAD` and read current `SPEC.md`, ADRs, source, and tests affected by the diff.
2. Ask `context-contracts` to analyze or update the existing Foreman contract convention.
3. Update SPEC only for changed current requirements, interfaces, invariants, or acceptance evidence.
4. Create an ADR only for an evidenced lasting decision with rationale and counterevidence. Avoid requiring one ADR per branch.
5. Preserve the established YAML metadata and decision numbering. When adding or changing a decision, update `docs/AGENTS.md`, `docs/README.md`, and the architecture decision index.
6. Run deterministic contract validation and final `context-review` after all context and prose edits stabilize.

Skip contract edits for formatting, tests that do not change accepted behavior, and dependency updates without a behavioral contract change. Never create placeholders or a separate decision ledger.
