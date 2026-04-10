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

- The repository now has `origin` configured at `safurrier/foreman`, and the
  1.0 PR is open as `#1`.

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

## 2026-04-10 10:55

- `cargo test --test claude_native -- --nocapture`
  - pass
- `cargo test --test runtime_dashboard -- --nocapture`
  - pass
- `cargo test --test notification_runtime -- --nocapture`
  - pass
- `cargo test --test tmux_actions -- --nocapture`
  - pass
- `cargo test`
  - pass
- `mise run check`
  - pass
- `mise run verify`
  - pass

Notes:
- The first GitHub Actions PR run failed on `ubuntu-latest` because the tmux
  E2E fixture and several runtime smoke tests assumed `zsh`. That caused pane
  commands to exit immediately on Linux and the tmux server to disappear before
  capture.
- The fix switched the shared tmux fixture and non-ignored runtime smoke tests
  to portable `sh`/`printf` command strings and corrected helper quoting so the
  keep-alive shell actually stays alive across platforms.
