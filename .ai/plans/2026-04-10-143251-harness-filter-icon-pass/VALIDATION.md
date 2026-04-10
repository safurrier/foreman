---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

- `cargo fmt`
- `cargo test --test tmux_inventory -- --nocapture`
- `cargo test --test runtime_dashboard -- --nocapture`
- `cargo test`
- `mise run verify-ux`
- `mise run check`
- `mise run verify`
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/verify_references.py .`
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/validate_frontmatter.py .`
