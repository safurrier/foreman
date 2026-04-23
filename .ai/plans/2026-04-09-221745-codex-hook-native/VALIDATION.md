---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

- `cargo test --test codex_hook_bridge --test codex_native -- --nocapture`
  Result: pass.
- `cargo test`
  Result: pass.
- `FOREMAN_REAL_CODEX_E2E=1 cargo test --test codex_real_e2e -- --ignored --nocapture`
  Result: pass. Verified the real `codex` binary emitted `submit` and `stop`
  hooks into `foreman-codex-hook`, which produced the expected native per-pane
  signal file.
- `mise run check`
  Result: pass.
- `mise run verify`
  Result: pass, including Docker build.
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/verify_references.py .`
  Result: pass.
- `python3 /Users/alex.furrier/.codex/skills/alex-ai-ai-context-engineering-files/scripts/validate_frontmatter.py .`
  Result: pass.

Notes:
- Local validation showed a real behavioral difference between Codex execution
  modes: `codex exec` emitted hooks reliably in supported non-interactive mode,
  while the same command inside a tmux TTY did not emit hooks in local testing.
  The real Codex-binary E2E therefore lives at the hook boundary, while tmux
  and dashboard behavior remain covered by Foreman's own runtime smoke tests.
