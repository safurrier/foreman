---
id: foreman-architecture
title: foreman Architecture
description: >
  Durable system record for foreman: invariants, principles, cross-cutting
  workflows, and architectural decisions. Read before implementing.
index:
  - id: system-overview
    keywords: [components, data-flows, trust-boundaries, modules]
  - id: invariants-boundaries
    keywords: [invariants, security, migrations, worktree-safety, observability, correlation-id, idempotent]
  - id: principles-patterns
    keywords: [thin-handlers, timeouts, structured-logs, typed-utilities, principles]
  - id: cross-cutting-workflows
    keywords: [validation-loop, check, verify, artifact-debugging, ci-failure]
  - id: decisions
    keywords: [adrs, decisions, truth-hierarchy, decisions-dir]
  - id: where-human-thought-goes
    keywords: [human-ownership, agent-ownership, promotion-rules]
---

# foreman — Architecture

> This document is the durable system of record for invariants, principles, and decisions.
> Keep it updated as the system evolves. Agents: read this before implementing.

## 1. System Overview

**Stack**: rust

### Components

<!-- List major components/modules and their responsibilities -->

| Component | Purpose |
|---|---|
| <!-- name --> | <!-- purpose --> |

### Primary Data Flows

<!-- Describe the main request/data flows through the system -->

1. <!-- flow 1 -->

### Trust Boundaries

<!-- Where does trust change? e.g., external API boundary, user input validation point -->

- <!-- boundary 1 -->

---

## 2. Goals / Non-Goals

**Goals** — what this system optimizes for:

- <!-- goal 1 -->

**Non-Goals** — explicit exclusions:

- <!-- non-goal 1 -->

---

## 3. Invariants & Boundaries

> These rules prevent catastrophic mistakes. Violating them requires an ADR.

### Security & Auth

- All authorization and critical validation occur in trusted service boundaries, not in clients.
- New endpoints or background jobs must declare their auth model before merging.

### Data & Migrations

- Database migrations are forward-only; no rollback scripts.
- Schema changes require a migration and a compatibility plan.

### Worktree Safety

- All services must start and all tests must pass from a **clean checkout or Git worktree**.
- No reliance on absolute paths, mutable global state, or undeclared local artifacts.
- Build artifacts must be deterministic or listed in `.gitignore`.

### Traceability & Observability

- All requests and background jobs carry a **correlation ID** (`trace_id` / `request_id`).
- Distributed calls propagate trace context.
- Logs are **structured** (machine-parseable) and include stable keys: `service`, `env`,
  correlation ID.
- Background jobs are **idempotent** and safe to retry.

> If these constraints do not yet apply (e.g., pure CLI, library), remove inapplicable
> sections and document why.

---

## 4. Principles & Preferred Patterns

> Durable preferences. If a rule is critical and objective, enforce it in CI instead.

- **Prefer thin handlers and service-layer logic** — reduces test cost and keeps handlers
  stateless.
- **Prefer explicit timeouts and retries on external calls** — prevents silent hangs.
- **Prefer structured logs with correlation IDs** — debugging depends on traceability.
- **Prefer shared typed utilities over one-off implementations** — invariants stay
  centralized.

<!-- Add project-specific principles here -->

---

## 5. Cross-Cutting Workflows

### Validation Loop

CI defines "passing." Local commands mirror CI exactly:

```bash
mise run check    # fast: fmt + lint + typecheck + unit tests  (on push)
mise run verify   # heavy: integration, security, docker        (on PR)
```

CI failures emit test artifacts to `test-results/` (uploaded by `actions/upload-artifact`).

### Artifact-First Debugging

When CI fails:

1. Open the GitHub Actions run summary → **Artifacts** → `test-results`.
2. **Python**: `test-results/junit.xml` — structured test output with failure details.
3. **Go**: `test-results/go-test.txt` — verbose test log.
4. Integration/docker failures: check logs in `test-results/`.

---

## 6. Decisions (ADRs)

Global architectural decisions live in `docs/decisions/`. Include only decisions that
materially constrain work.

**Truth hierarchy** (highest to lowest authority):

1. CI/tooling enforcement
2. ADRs in `docs/decisions/`
3. This document (`docs/architecture.md`)
4. Module-level docs and `AGENTS.md`

**Index**:

| ADR | Decision |
|---|---|
| [0001-stack-choice](decisions/0001-stack-choice.md) | Stack selection for foreman |

---

## 7. Module Map

<!-- For single-project repos, list major internal modules.
     For apps workspace, list apps and packages with links. -->

| Module | Purpose | Docs |
|---|---|---|
| <!-- module --> | <!-- purpose --> | — |

---

## 8. Where Human Thought Goes

**Humans define**:

- Invariants and boundaries (section 3)
- Decision records (ADRs)
- What gets enforced in CI/tooling
- Observability minimums
- Worktree compatibility rules

**Agents own**:

- Implementation following these constraints
- Completing the validation loop (`mise run check` must pass before committing)

**Promotion rules**:

- Repeated ambiguity → document here or in module docs
- Repeated objective failure → encode in CI/tooling
- Repeated preference debate → optional Skill plugin (`.agent/skills/`)
