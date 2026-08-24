---
name: spec-sync
description: Route a branch diff through the repository's current SPEC and ADR contract owners. Use when reviewing whether changed behavior requires a contract update before handoff.
allowed-tools: Read, Edit, Write, Glob, Grep, Bash
---

# Contract sync compatibility route

Use the installed `context-contracts` workflow as the preferred authority for `SPEC.md` and `docs/decisions/`. This repo-local skill supplies Foreman routing plus a portable fallback for fresh checkouts where that external skill is unavailable.

## Workflow

1. Inspect `git diff origin/main...HEAD` and read current `SPEC.md`, ADRs, source, and tests affected by the diff.
2. When `context-contracts` is available, ask it to analyze or update the existing Foreman contract convention.
3. Otherwise, apply the portable fallback:
   - Keep exactly the six existing second-level SPEC sections and edit only changed requirements, interfaces, invariants, or acceptance evidence.
   - Create an ADR only for an evidenced lasting decision with rationale and counterevidence. Avoid requiring one ADR per branch.
   - Preserve the established YAML metadata, original decision date, update history, and sequential numbering.
4. When adding or changing a decision, update `docs/AGENTS.md`, `docs/README.md`, and the architecture decision index.
5. Run `tests/docs_contract_test.py` through the repository Python environment and run `git diff --check`.
6. When `context-review` is available, run it after all context and prose edits stabilize. Otherwise, report that semantic context review was unavailable rather than inventing an equivalent check.

Skip contract edits for formatting, tests that do not change accepted behavior, and dependency updates without a behavioral contract change. Never create placeholders or a separate decision ledger.
