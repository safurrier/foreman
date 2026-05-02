---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — context-improve-refresh

## Problem

Run a context-improvement pass on foreman using the lean context-engineering
plugin guidance. The current root `AGENTS.md` is useful but slightly over the
repo-root target size, and validation reports missing context-engineering
watermarks plus a few false path references from backticked prose.

## Requirements

### MUST

- Preserve the existing foreman workflow contracts: `SPEC.md` remains the
  product contract, `docs/architecture.md` remains the architecture record, and
  plan artifacts remain the slice memory.
- Keep root `AGENTS.md` lean and cross-cutting; move or point to detailed docs
  instead of repeating them.
- Fix context-engineering reference-check failures caused by backticked prose
  that is not meant to be a path.
- Add generated-by watermarks expected by the context-engineering validator.
- Validate with the context-engineering CLI and the repo's fast gate.

### SHOULD

- Avoid creating new docs unless the lean context pass exposes a real gap.
- Keep command guidance only where it is validated and useful to future agents.

## Constraints

- Do not change product behavior.
- Do not run the optional structured docs flow; foreman already has frontmattered
  docs.
- No relevant prior sessions were found by the context-engineering session
  scanner, so derive changes from existing repo context and deterministic checks.
