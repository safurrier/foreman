---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-10 09:15 - CI already existed, but the real 1.0 gap was CD

The repo already had `ci.yml`, so the problem was not "set up GitHub Actions"
from scratch. The real gap was that there was no release workflow at all and
the crate was still versioned `0.1.0`. That changed the task from CI setup to a
release-automation pass with versioning and packaging.

## 2026-04-10 09:20 - Release bundles need the hook binaries too

It would have been easy to ship only `foreman`, but that would make native
Claude, Codex, and Pi support incomplete the moment someone downloaded a
release archive. The release package needs to include `foreman-claude-hook`,
`foreman-codex-hook`, and `foreman-pi-hook` as first-class release artifacts.

## 2026-04-10 10:05 - Actionlint caught a real runner-label drift

The first release workflow draft used `macos-13` for the Intel macOS build.
`actionlint` flagged that as an outdated label in its current runner catalog, so
the workflow moved to `macos-15-intel`. That was a good reminder that workflow
syntax review is not enough; the runner catalog changes underneath you.

## 2026-04-10 10:25 - The 1.0 version bump exposed a Docker cache weakness

`mise run verify` broke after the crate version moved to `1.0.0`, not because
the app stopped building, but because the Dockerfile tied the dependency-cache
layer directly to `Cargo.toml`. A version bump invalidated that layer, forced a
fresh dependency build inside Docker, and exposed unreliable network resolution
to `crates.io` in this Colima environment.

The fix was to add a stable `Cargo.docker.toml` dependency manifest for the
cache-priming step and then copy the real manifest and source later. That made
the heavy gate stable again and also let the container image preserve the real
`foreman` command name plus the hook binaries.

## 2026-04-10 10:30 - PR creation is blocked only by repo plumbing

The product and release automation work are ready for a PR, but this checkout
has no git remote configured. That means push and `gh pr create` are blocked by
repository plumbing, not by code or validation state.
