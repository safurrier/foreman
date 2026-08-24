---
id: foreman-adr-0001
title: ADR 0001—Stack Choice for foreman
description: >
  Records the rust stack selection decision for foreman,
  including rationale, trade-offs, and alternatives considered.
status: accepted
date: 2026-04-08
index:
  - id: decision
    keywords: [stack, choice, python, go, tools, rationale]
  - id: consequences
    keywords: [trade-offs, positive, negative, accepted]
  - id: alternatives-considered
    keywords: [alternatives, rejected, comparison]
---

# ADR 0001: Stack Choice for foreman

**Generated from**: init

---

## Context

foreman requires a primary implementation stack for building, testing, and
deploying the application. The choice constrains tooling, CI configuration, and
contributor onboarding.

## Decision

**Stack**: rust

The Rust stack uses:
- **cargo fmt** for formatting (rustfmt under the hood)
- **cargo clippy** for linting (hundreds of lint rules)
- **cargo check** for fast type/borrow checking
- **cargo test** for testing

## Consequences

**Positive**:

- Standard tooling with strong ecosystem support.
- Consistent quality gates via `mise run check`.
- Reproducible builds via mise tool version pinning.

**Negative / Trade-offs**:

- Cold builds and the heavy validation gate are slower than the normal edit loop.

## Alternatives Considered

| Alternative | Reason not chosen |
|---|---|
| Python | Faster iteration, but weaker compile-time guarantees for a stateful terminal control plane. |
| Go | Simple deployment, but less alignment with the chosen Ratatui ecosystem and reducer-oriented UI implementation. |
