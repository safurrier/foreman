---
id: foreman-spec
title: foreman Specification
description: >
  Correctness envelope — requirements, contracts, and invariants
  for foreman.
index:
  - id: requirements
    keywords: [must, should, may, behavioral, requirements]
  - id: interfaces
    keywords: [api, contracts, boundaries, modules, io]
  - id: invariants
    keywords: [state, safety, data, rules, boundaries, always-true]
---

# foreman — Specification

> This document defines the correctness envelope: what must be true about any
> valid implementation. For how the system works now, see `docs/architecture.md`.
> For how to work in this repo, see `AGENTS.md`.

## Summary

<!-- What this project is, in one paragraph. -->

## Goals / Non-Goals

**Goals:**

- <!-- What the system optimizes for -->

**Non-Goals:**

- <!-- Explicit exclusions — what this project will NOT do -->

## Requirements

<!-- Behavioral requirements using normative language. -->
<!-- For small projects: list requirements directly. -->
<!-- For larger projects: link to docs/requirements/ for module-specific requirements. -->

### MUST

- `mise run check` passes on a freshly initialized project without manual intervention
- `mise run ci` produces identical results to `mise run check`

### SHOULD

- <!-- Expected but not strictly required behaviors -->

### MAY

- <!-- Optional behaviors -->

## Interfaces & Contracts

<!-- Public APIs, module boundaries, I/O shapes, CLI surfaces. -->
<!-- For small projects: describe inline. -->
<!-- For larger projects: link to API specs, OpenAPI docs, or module contracts. -->

**Task contract:**

| Command | Purpose |
|---------|---------|
| `mise run check` | Fast quality gate (fmt + lint + typecheck + test) |
| `mise run verify` | Heavy validation (integration, docker, security) |
| `mise run ci` | CI entrypoint (delegates to `check`) |

## Invariants

<!-- Rules that must always hold — violating these requires an ADR. -->
<!-- Heuristic: if CI can prove or falsify it, it belongs here. -->
<!-- For module-scoped invariants in larger projects, link to apps/<mod>/SPEC.md -->

- All services must start and all tests must pass from a **clean checkout or Git worktree**.
- CI and local checks use the **same entrypoints** — no divergence.

## Acceptance

<!-- How to verify this spec is satisfied. Points to test commands and CI gates. -->

```bash
mise run check    # fast: fmt + lint + typecheck + test
mise run verify   # heavy: integration, e2e, docker, security
```
