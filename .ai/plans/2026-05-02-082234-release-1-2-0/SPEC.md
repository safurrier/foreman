---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — release-1-2-0

## Problem

Foreman main now contains user-visible native harness fixes and macOS
notification sound support. The repo needs release metadata and release notes
for version 1.2.0 before tagging.

## Requirements

### MUST

- Bump the crate version from 1.1.1 to 1.2.0.
- Add a changelog entry covering the native and notification changes.
- Update README version metadata.
- Run release validation before opening the release PR.

### SHOULD

- Keep the release PR limited to metadata, release notes, and validation
  evidence.
- Use the existing release workflow: merge release PR, then tag `v1.2.0`.

## Constraints

- Do not tag until the release PR is merged to `main`.
- Do not mix new product changes into the release PR.
