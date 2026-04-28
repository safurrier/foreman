---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — 1-1-0-release-doc-sync

## Problem

Foreman is ready for a `1.1.0` tag, but the release branch should first make
the release-facing docs match the shipped state. The current crate version is
already `1.1.0`, while the changelog still stops at `1.0.0`.

## Requirements

### MUST

- Add a `1.1.0` changelog entry that summarizes the merged release work.
- Keep `SPEC.md` aligned with shipped native integration status.
- Keep the release process docs clear about changelog updates before tagging.
- Validate locally before opening the PR.
- After the PR passes and merges, tag `v1.1.0` from updated `main`.

### SHOULD

- Keep edits focused on release hygiene and avoid product-code changes.
- Preserve existing documentation tone and structure.

## Constraints

- The release tag must match `Cargo.toml` exactly or the release workflow fails.
- The GitHub release workflow performs the final verify/build/publish work after
  the tag is pushed.
