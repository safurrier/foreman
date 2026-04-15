---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — native-e2e-hardening

## Approach

Treat this as two problems:

1. Environment truth and operator guidance
2. Test/harness correctness on a ready machine

The repo changes should make PATH/version/auth mismatches obvious before the
real suite runs, then keep the actual real suite mandatory for closeout. The
Codex lane needs an explicit hook-support floor. The Claude lane needs a timing
stability pass so strict native verification can actually pass on a configured
machine.

## Steps

1. Record the current failure modes in the plan and validation log.
2. Add `native-preflight` under `.mise/tasks/` with tmux, CLI, and Codex hook
   readiness checks.
3. Tighten `verify-native` to support strict mode and invoke preflight for the
   selected providers before running the real tests.
4. Add a Codex version/feature guard to `tests/codex_real_e2e.rs`.
5. Stabilize `tests/claude_real_e2e.rs` so the completion notification proof is
   not timing-fragile.
6. Update docs and agent guidance around strict closeout and preflight usage.
7. Run focused real E2Es, then `mise run check`, then strict `verify-native`.
