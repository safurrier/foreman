---
id: foreman-adr-0001
title: ADR 0001 — Stack Choice for foreman
description: >
  Records the rust stack selection decision for foreman,
  including rationale, trade-offs, and alternatives considered.
index:
  - id: decision
    keywords: [stack, choice, python, go, tools, rationale]
  - id: consequences
    keywords: [trade-offs, positive, negative, accepted]
  - id: alternatives-considered
    keywords: [alternatives, rejected, comparison]
---

# ADR 0001: Stack Choice for foreman

**Status**: Accepted
**Date**: <!-- YYYY-MM-DD -->
**Deciders**: <!-- names or team -->
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

<!-- List stacks that were considered but not chosen, and why -->

| Alternative | Reason not chosen |
|---|---|
| <!-- alt --> | <!-- reason --> |
