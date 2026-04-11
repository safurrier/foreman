---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

- `go run github.com/rhysd/actionlint/cmd/actionlint@latest` — pass
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/verify_references.py .` — pass
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/validate_frontmatter.py .` — pass
- `cargo test --lib ui::render::tests -- --nocapture` — pass
- `cargo test --test runtime_dashboard -- --nocapture` — pass
- `cargo test --test release_gauntlet -- --test-threads=1 --nocapture` — pass
- `mise run verify-ux` — pass
- `mise run verify-release` — pass
- `mise run check` — pass
- `mise run verify` — pass
