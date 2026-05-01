---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

## Automated

- `cargo fmt --check` — passed.
- `cargo test -q apply_native_signals_does_not_override_different_harness -- --nocapture` — passed.
- `cargo test -q apply_native_signals_can_correct_compatibility_mislabel_without_runtime_identity -- --nocapture` — passed.
- `cargo test -q codex_native_signal_corrects_compatibility_mislabel_without_runtime_identity -- --nocapture` — passed.
- `mise run check` — passed.
