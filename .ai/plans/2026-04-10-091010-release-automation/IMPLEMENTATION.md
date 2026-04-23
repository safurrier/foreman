---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — release-automation

## Approach

Keep CI as-is and add the missing CD layer separately. The release path will be
tag-driven, not branch-driven: a `v*` tag should re-run heavy validation on
Linux, then build release bundles on the native GitHub-hosted runners we want
to support, and finally publish a GitHub release with those archives.

The packaging decision is important: 1.0 needs to ship the companion hook
binaries too, not just `foreman`, because native Claude, Codex, and Pi
integrations depend on them.

## Steps

1. Bump crate version and add release-facing docs.
2. Add a tag-driven GitHub Actions release workflow with version/tag checks.
3. Package `foreman` plus `foreman-claude-hook`, `foreman-codex-hook`, and
   `foreman-pi-hook` into release archives.
4. Validate workflow syntax locally with `actionlint`.
5. Run `cargo test`, `mise run check`, `mise run verify`, and doc validators.
6. Record the remote/PR blocker explicitly if no git remote exists locally.
