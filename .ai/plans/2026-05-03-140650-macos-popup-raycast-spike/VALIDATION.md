---
id: macos-popup-raycast-spike-validation
title: Validation Log
description: >
  Evidence gathered during the macOS popup/Raycast speclab pass.
---

# Validation — macos-popup-raycast-spike

## 2026-05-03 — Research/spec validation

Commands run:

```bash
git status --short --branch
mise run plan -- macos-popup-raycast-spike
find src -maxdepth 3 -type f | sort
rg -n "json|serde_json|Command|Subcommand|Args|Parser|focus|send|tmux|popup" src tests Cargo.toml README.md SPEC.md docs/architecture.md
```

Code/doc evidence reviewed:

- `SPEC.md` — product contract for tmux/popup behavior.
- `docs/architecture.md` — adapter/reducer/rendering boundaries.
- `src/cli.rs` — current clap parser, JSON doctor precedent, bootstrap flow.
- `src/adapters/tmux.rs` — inventory/focus/send adapter seams.
- `src/app/state.rs` — inventory summary and visible/actionable target cache.
- `docs/operator-guide.md` — current tmux popup and macOS notification behavior.

External research sources:

- Raycast Create Your First Extension: https://developers.raycast.com/basics/create-your-first-extension
- Raycast List API: https://developers.raycast.com/api-reference/user-interface/list
- Raycast useExec API: https://developers.raycast.com/utilities/react-hooks/useexec
- CLI Guidelines output section: https://clig.dev/#output

No Rust code was changed in this pass, so `mise run check` was not run. The
output of this slice is the plan/spec artifacts themselves.
