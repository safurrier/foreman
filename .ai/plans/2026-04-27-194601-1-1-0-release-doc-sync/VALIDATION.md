---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

- `python /Users/alex.furrier/.codex/skills/alex-ai-docs-workflow/scripts/docs_verify.py /Users/alex.furrier/git_repositories/foreman` — failed because `python` resolved to an interpreter that cannot parse modern type syntax.
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-docs-workflow/scripts/docs_verify.py /Users/alex.furrier/git_repositories/foreman` — passed.
- `mise run check` — passed.
- `mise run verify-release` — passed and refreshed `.ai/validation/release/`.
