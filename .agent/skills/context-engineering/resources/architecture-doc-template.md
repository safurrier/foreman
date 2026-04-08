# Architecture Document Template

Template for generating `docs/architecture.md`. This is the durable system record
for invariants, principles, and architectural decisions. Agents should read this
before implementing significant changes.

## Contents
- When to generate
- Frontmatter template
- Sections (system overview, invariants, principles, workflows, truth hierarchy, module map)
- Discovery strategy
- Sizing guidelines

## When to Generate

- Auto mode: repos with 3+ modules or significant cross-cutting complexity
- On explicit request
- When invariants or cross-cutting patterns emerge that don't fit in AGENTS.md

## Frontmatter

```yaml
---
id: {project}-architecture
title: {Project} Architecture
description: >
  System record: invariants, principles, cross-cutting workflows, and decisions.
  Read before implementing significant changes.
index:
  - id: system-overview
  - id: invariants-boundaries
  - id: principles-patterns
  - id: cross-cutting-workflows
  - id: truth-hierarchy
---
```

## Sections

### 1. System Overview

- **Stack**: Language(s), framework(s), key tools
- **Components**: Table of major components and their responsibilities
- **Primary data flows**: How requests/data move through the system
- **Trust boundaries**: Where trust changes (external API boundary, user input, etc.)

### 2. Goals / Non-Goals (optional)

Include if discoverable from README, docs, or comments.

- **Goals**: What this system optimizes for
- **Non-Goals**: Explicit exclusions

### 3. Invariants & Boundaries

More detailed than the AGENTS.md invariants section. Categorized:

**Security & Auth:**
- Authorization and critical validation occur in trusted boundaries, not clients
- New endpoints declare their auth model before merging

**Data & Migrations:**
- Migrations are forward-only; no rollback scripts
- Schema changes require a migration and compatibility plan

**Build & CI:**
- All tests pass from a clean checkout or git worktree
- No reliance on absolute paths or undeclared local artifacts
- Build artifacts are deterministic or listed in `.gitignore`

**Runtime (if applicable):**
- Background jobs are idempotent and safe to retry
- Distributed calls propagate trace context

> Mark sections that don't apply (e.g., pure CLI/library) and note why.

### 4. Principles & Preferred Patterns

Durable preferences discovered from code and config. Format:

- **Prefer X over Y** — {rationale}

Examples:
- **Prefer thin handlers and service-layer logic** — reduces test cost, keeps handlers stateless
- **Prefer explicit timeouts on external calls** — prevents silent hangs
- **Prefer shared typed utilities over one-off implementations** — centralizes invariants

> If a principle is critical and objective, it should be enforced in CI instead
> of documented here.

### 5. Cross-Cutting Workflows

**Validation loop:**
```
{fast-check-command}    # fast: fmt + lint + typecheck + test  (before committing)
{heavy-verify-command}  # heavy: integration, security, docker  (before merging)
```

CI calls the same commands. If local check passes, CI passes.

**Debugging workflows:**
- Where to find CI artifacts
- How to reproduce failures locally

### 6. Truth Hierarchy

Explicitly document authority (highest to lowest):

1. **CI/tooling enforcement** — automated, authoritative
2. **Decision records** (ADRs in `docs/decisions/` if they exist)
3. **This document** (`docs/architecture.md`)
4. **AGENTS.md** files

**Promotion rules:**
- Repeated ambiguity → document it here or in module docs
- Repeated objective failure → encode in CI/tooling
- Repeated preference debate → optional convention or skill

### 7. Module Map (for multi-module repos)

| Module | Purpose | Docs |
|--------|---------|------|
| {module} | {purpose} | {link to nested AGENTS.md} |

## Discovery Strategy

To populate the architecture doc:

1. **CI workflows** → enforced rules become invariants
2. **Linter/formatter configs** → conventions and principles
3. **Test structure** → validation patterns and workflows
4. **Package manifests** → dependency graph and module relationships
5. **Existing docs/README** → stated principles and goals
6. **Code structure** → trust boundaries and data flows

## Sizing

- Target: 80-150 lines for typical single-project repos
- Allow up to 200 lines for complex monorepos
- If it would exceed 200, split into `docs/architecture/` subfolder
