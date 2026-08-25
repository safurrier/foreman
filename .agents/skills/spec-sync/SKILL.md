---
name: spec-sync
description: Check whether a branch changes the Foreman specification or a decision record. Use before handoff when behavior or public contracts changed.
allowed-tools: Read, Edit, Write, Glob, Grep, Bash
---

# Contract sync route

Use `context-contracts` when installed. If missing, use the local steps below.

## Workflow

1. Read `git diff origin/main...HEAD`, `SPEC.md`, the decision records, and related tests.
2. Change the specification only when the branch changes a current rule, interface, invariant, or proof.
3. Add a decision record only for a lasting choice with clear reasons and tradeoffs.
4. Keep the six specification sections, YAML fields, original decision dates, update history, and file numbering.
5. Update `docs/AGENTS.md`, `docs/README.md`, and the decision table in `docs/architecture.md` when a decision changes.
6. Run `tests/docs_contract_test.py` through the repo's Python tool and run `git diff --check`.
7. Run `context-review` when installed. If missing, report that gap.

Skip edits for formatting, tests that don't change behavior, and dependency updates with no contract effect. Never create placeholder records or a second decision index.
