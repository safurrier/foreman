---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — release-automation

## Problem

The repo already has a GitHub Actions CI workflow for `main` pushes and pull
requests, but it does not yet have a tag-driven release workflow and the crate
still reports version `0.1.0`. The user wants this repo prepared for the real
1.0 release path, including GitHub Actions automation and a merge-ready branch.

## Requirements

### MUST

- Keep the existing `ci.yml` behavior for fast-gate and PR verification intact.
- Add a GitHub Actions release workflow that runs on `v*` tags.
- The release workflow must verify the repo before publishing artifacts.
- The release workflow must publish archives that include the main `foreman`
  binary and all shipped hook bridge binaries.
- The release workflow must fail if the pushed tag does not match the crate
  version in `Cargo.toml`.
- The crate version must be bumped to `1.0.0`.
- Human-facing docs must describe the CI and release flow accurately.
- Plan artifacts and validation logs must be updated as part of the change.

### SHOULD

- Release artifacts should cover the main target runners available in GitHub
  Actions without introducing fragile cross-compilation.
- Release notes should be easy to generate from the GitHub release flow.
- Local validation should include a workflow linter rather than trusting YAML by
  inspection alone.

## Constraints

- Do not break the existing `mise run check` or `mise run verify` contract.
- Do not guess a GitHub remote URL; PR creation is blocked until a remote exists
  in this checkout.
- Keep release packaging simple and explicit rather than adding a large release
  toolchain dependency.
