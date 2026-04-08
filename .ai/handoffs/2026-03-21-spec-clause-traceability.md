# Handoff: Spec Clause Traceability for agent-scaffold

**Date**: 2026-03-21
**Status**: Design complete, not yet implemented
**Repo**: `/Users/alex.furrier/git_repositories/agent-scaffold`

---

## Origin Story

This came out of a conversation about what the "Turbo Pascal moment" for AI coding tools looks like — the point where the environment around the agent becomes the primary lever, not the model. Three ideas were converging:

1. **Harness engineering** (OpenAI/Codex framing): constraints encoded mechanically (linters, structural tests) beat constraints encoded by convention. Linter errors written as remediation instructions. Observability fed back into agent context.

2. **Spec-driven development** (GitHub Next SpecLang, spec-kit): the spec is the source of truth, code is last-mile output. Maintaining software means evolving the spec. Debugging means fixing the spec.

3. **Intent-verification architecture** (the synthesis): invert the loop. Instead of humans iterating through code, the system continuously verifies code against human intent. Two checking layers:
   - **Deterministic**: linters, type checks, structural tests — fast, always-on
   - **Semantic (LLM-based)**: "does the codebase satisfy the spec intent?" — triggered on meaningful state changes, outputs confidence per spec clause

The human's job becomes maintaining intent. Drift surfaces proactively; agents propose repairs; humans approve or refine the spec.

## The Specific Insight

agent-scaffold already has most of the harness — SPEC.md with requirements, contract tests that validate structure, `_docs_helpers.py` with stdlib-only parsers, a spec-sync skill that reviews diffs against SPEC.md, plan templates with TODO.md and VALIDATION.md.

The missing piece is **traceability** — there's no way to mechanically link a requirement to the test that verifies it, or to the plan item that implements it. Everything is connected by human reading comprehension, not by machine-checkable references. This means:

- You can't answer "which requirements are untested?" without reading every test
- You can't answer "does this plan item trace to a requirement?" without reading SPEC.md
- You can't detect when a spec change invalidates linked tests
- An LLM-based semantic check has to evaluate the *entire* codebase against the *entire* spec rather than targeted clause-by-clause verification

The fix is simple: give every requirement a stable ID, make tests and plans cite those IDs, and add mechanical checks that enforce the links.

## The Dev Experience We Were Imagining

A **plan → execute → verify** workflow with ambient verification across all three phases:

- **During planning**: semantic diff of proposed plan against spec catches intent conflicts before execution
- **During execution**: a live confidence gauge tracking agent trajectory against spec. Yellow = agent making spec-uncovered decisions (flag, don't stop). Red = agent contradicting a spec clause (stop and surface)
- **During verification**: not just "did tests pass" but "does this implementation satisfy the stated intent?" — a diff at the intent level

The clause traceability system is the foundation that makes all of this mechanically possible. Without clause IDs, the semantic layer has no anchors.

---

## Current State of the Repo

### What already exists

| Component | Where | What it does |
|-----------|-------|-------------|
| SPEC.md | `SPEC.md` | 21 requirements (14 MUST, 4 SHOULD, 3 MAY) as plain bullets — no IDs |
| Spec template | `templates/SPEC.md.tmpl` | Jinja2 template for generated projects, 2 starter MUST bullets |
| Doc parsers | `tests/_docs_helpers.py` | stdlib-only parsers: frontmatter, sections, ADRs, META.yaml. ~400 lines, regex-based, zero deps |
| Contract tests | `tests/contract/test_task_contract.py` | Validates 13-task contract (file exists, executable, shebang, description header) |
| Doc contract tests | `tests/contract/test_docs_contract.py` | Validates frontmatter, sections, ADR structure, SPEC template sections |
| Pytest markers | `pyproject.toml` | 6 markers: contract, unit, e2e, slow, go, rust |
| spec-sync skill | `.agent/skills/spec-sync/SKILL.md` | Pre-push: reviews diffs against SPEC.md, proposes ADRs/updates |
| Plan templates | `templates/.ai/plans/_templates/` | META.yaml, TODO.md, SPEC.md, LEARNING_LOG.md, VALIDATION.md |
| Stack protocol | `src/agent_scaffold/stacks/base.py` | Structural interface for stack implementations |

### What's missing (the gap this design fills)

- No clause IDs on requirements — can't reference specific requirements
- No test-to-requirement linking — can't tell which tests verify which requirements
- No plan-to-requirement linking — can't tell which TODO items implement which requirements
- No coverage report — can't answer "what's untested?"
- No orphan detection — can't catch tests referencing removed requirements
- spec-sync skill has no clause-level awareness

---

## The Design

### Clause ID Format

`[LEVEL-NNN]` — brackets, normative level, three-digit zero-padded number.

```markdown
- [MUST-001] `mise run init` supports interactive and non-interactive modes
- [SHOULD-001] `mise run check` completes in under 60 seconds
```

IDs are permanent. Removing a requirement retires the ID — never renumber. This is critical: tests, plans, and ADRs reference these IDs, and renumbering would silently break all references.

Regex: `\[(?:MUST|SHOULD|MAY)-\d{3}\]`

### Data Model

```python
@dataclass
class SpecClause:
    id: str          # "MUST-001" (without brackets)
    level: str       # "MUST" | "SHOULD" | "MAY"
    number: int      # 1
    text: str        # requirement text after the ID
    line_number: int  # 1-indexed in the source file
```

### New Functions in `_docs_helpers.py`

| Function | Purpose |
|----------|---------|
| `parse_spec_clauses(path)` | Parse `[LEVEL-NNN]` bullets from SPEC.md → `list[SpecClause]` |
| `validate_spec_clauses(clauses, *, allow_gaps=False)` | Check uniqueness, sequentiality, ordering → `list[str]` errors |
| `find_untagged_requirements(path)` | Find bullets under MUST/SHOULD/MAY without clause IDs |
| `collect_clause_coverage(spec_path, test_dir, plans_dir)` | Cross-reference clauses against `@pytest.mark.covers` and TODO refs |
| `parse_todo_clause_refs(path)` | Parse `[LEVEL-NNN]` references from TODO.md checkboxes |
| `format_coverage_report(coverage)` | Render coverage matrix as markdown table |

All stdlib-only. All follow the existing `parse_X` / `validate_X` separation pattern.

### Contract Tests: `tests/contract/test_spec_clauses.py`

| Test | Asserts |
|------|---------|
| `test_scaffold_spec_has_clauses` | At least one clause exists |
| `test_scaffold_spec_clauses_are_valid` | No duplicate IDs, sequential, ordered |
| `test_scaffold_spec_no_untagged_requirements` | Every bullet under MUST/SHOULD/MAY has an ID |
| `test_spec_template_has_clause_ids` | Generated SPEC.md template uses clause IDs |

### Test-to-Clause Linking

`@pytest.mark.covers("MUST-001")` marker. Registered in pyproject.toml. Discovered via static regex scan (no pytest runtime needed).

### Plan-to-Clause Linking

TODO.md items reference clause IDs: `- [x] [MUST-008] Add JWT validation middleware`

### Coverage Report

`scripts/spec_coverage.py` — prints a markdown table showing which clauses have tests, which have plan refs, and which are uncovered. Supports `--json`.

### Spec-Sync Skill Updates

New "Maintain clause IDs" step: assign next sequential ID to new requirements, retire (don't renumber) removed ones, report new IDs assigned.

---

## Design Tradeoffs & Decisions

### ID stability over compactness
IDs are never reused or renumbered. This creates gaps in evolved specs but preserves referential integrity. The validator has an `allow_gaps` parameter: `False` for fresh specs, `True` for evolved specs.

### Level in the ID vs. separate namespace
We put MUST/SHOULD/MAY in the ID itself (`MUST-001` not `REQ-001`). This means criticality is visible at every reference site without looking up the spec. Tradeoff: if a requirement changes level (SHOULD → MUST), the ID becomes misleading. Decision: this is rare enough that manual re-ID is acceptable, and the contract tests will catch the inconsistency.

### Regex discovery vs. pytest plugin
Coverage discovery uses static regex scanning of test files, not a pytest plugin. This keeps it stdlib-only and usable without running pytest. Tradeoff: regex can't handle dynamic marker construction (e.g., `mark = pytest.mark.covers; mark("MUST-001")`). Decision: that pattern should never exist in practice.

### `allow_gaps` dual mode
Fresh specs enforce strict sequentiality (no gaps). Evolved specs allow gaps from retired clauses. The scaffold's own SPEC.md starts strict. Generated projects use `allow_gaps=True` since they'll evolve. This avoids the false choice between "renumber everything" and "never enforce ordering."

### Coverage enforcement scope
Only MUST clauses require test coverage. SHOULD produces warnings. MAY has no enforcement. Matches normative semantics — a SHOULD without a test is a smell, not a failure.

### No separate mapping file
Clause IDs live inline in the SPEC.md bullets, not in a separate `clause-map.yaml`. This adds ~11 chars of visual noise per bullet but keeps the single-source-of-truth property. A mapping file would drift from the actual requirements.

---

## Implementation Sequence

### Minimal viable slice (get the IDs and structural checks working)
1. Add `SpecClause` dataclass + `parse_spec_clauses()` + `validate_spec_clauses()` + `find_untagged_requirements()` to `_docs_helpers.py`
2. Add clause IDs to all 21 bullets in `SPEC.md`
3. Add `tests/contract/test_spec_clauses.py`
4. `mise run check` — verify green
5. Add clause IDs to `templates/SPEC.md.tmpl`

### Full system (close the traceability loop)
6. Register `covers` marker in `pyproject.toml`
7. Add `@pytest.mark.covers(...)` to existing tests (mechanical backfill)
8. Add `parse_todo_clause_refs()`, `collect_clause_coverage()`, `format_coverage_report()` to `_docs_helpers.py`
9. Add `scripts/spec_coverage.py`
10. Update `spec-sync` skill, update example plan TODO.md

### Verification criteria
- `mise run check` green (existing + new contract tests)
- `parse_spec_clauses()` returns 21 clauses, `validate_spec_clauses()` returns no errors
- `grep -r 'MUST-001' tests/` returns hits
- `uv run python scripts/spec_coverage.py` produces a complete matrix
- Every MUST clause has at least one `@pytest.mark.covers` reference

---

## Broader Vision (Not In Scope, But Motivating)

The clause traceability system is the foundation for the intent-verification architecture. Once every requirement has an ID and tests cite those IDs:

- **Targeted semantic verification**: Instead of "does the whole codebase satisfy the whole spec," an LLM evaluates "does the code linked to MUST-003 actually satisfy MUST-003?" This is cheaper, more reliable, and produces actionable output.

- **Live intent health panel**: A dashboard showing spec clauses with verification status, active plan items mapped against spec, constraint violations, semantic drift signals. Human resolves drift by editing the spec.

- **Spec-level diffs**: When a PR changes code, the system identifies which clause IDs are affected (via `covers` markers on changed tests) and runs targeted semantic checks on just those clauses.

- **Agent trajectory monitoring**: During plan execution, the system tracks which clause IDs an agent is working on and flags when it makes decisions not covered by any clause (yellow) or contradicting a clause (red).

None of this is possible without the clause ID anchors. The traceability system is step zero.

---

## Key Files Reference

| File | Lines | What to know |
|------|-------|-------------|
| `SPEC.md` | 1-139 | Current requirements, frontmatter with keyword index |
| `tests/_docs_helpers.py` | 1-403 | All parsers. `parse_sections` (155), `parse_frontmatter` (32), `parse_meta_yaml` (362) |
| `tests/contract/test_task_contract.py` | 1-229 | Pattern for contract tests. 13-task validation |
| `tests/contract/test_docs_contract.py` | 1-284 | Pattern for doc structure tests. Frontmatter, ADR, SPEC template |
| `templates/SPEC.md.tmpl` | 1-86 | Generated project SPEC.md template |
| `templates/.ai/plans/_templates/` | — | Plan artifact templates |
| `.agent/skills/spec-sync/SKILL.md` | 1-119 | Pre-push spec review skill |
| `src/agent_scaffold/stacks/base.py` | 1-42 | Stack protocol (context for understanding the codebase) |
| `pyproject.toml` | 42-53 | Pytest config, marker registration |

---

## How to Pick This Up

1. Read this doc
2. Read the design plan at `~/.claude/plans/enumerated-noodling-bentley.md` for implementation-level detail
3. Start with the minimal slice (steps 1-5) — it's self-contained and verifiable
4. Run `mise run check` after each step to confirm nothing breaks
5. The full system (steps 6-10) builds on the minimal slice with no rework
