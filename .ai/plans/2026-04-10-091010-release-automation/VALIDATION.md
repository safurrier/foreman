---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

## Planned validation stack

- `go run github.com/rhysd/actionlint/cmd/actionlint@latest`
- `cargo test`
- `mise run check`
- `mise run verify`
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/verify_references.py .`
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/validate_frontmatter.py .`

## Notes

- This checkout has no git remote configured, so GitHub PR creation cannot be
  completed locally until a remote exists.

## 2026-04-10

- `go run github.com/rhysd/actionlint/cmd/actionlint@latest`
  - pass
- `cargo test`
  - pass
- `mise run check`
  - pass
- `mise run verify`
  - pass
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/verify_references.py .`
  - pass
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/validate_frontmatter.py .`
  - pass

Notes:
- The first `actionlint` pass failed because `macos-13` is no longer a
  recognized hosted-runner label; the workflow now uses `macos-15-intel`.
- The first `mise run verify` pass failed after the version bump because the
  Docker dependency-cache layer depended directly on `Cargo.toml`. Adding
  `Cargo.docker.toml` fixed the cache invalidation path and restored a green
  Docker build.
