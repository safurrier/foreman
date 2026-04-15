---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

- 2026-04-12 12:39 - The first real blocker was not “Codex hooks broke” but a
  PATH mismatch. `npm list -g` showed `@openai/codex@0.120.0`, while `codex
  --version` resolved to `0.92.0` from `~/.local/bin`. The repo needs to gate
  against the PATH-resolved binary because that is what the tests actually run.
- 2026-04-12 12:40 - The current official Codex hook docs still describe hooks
  behind `features.codex_hooks=true`, so the repo should not remove hook-based
  Codex support. It should require a new enough CLI instead.
- 2026-04-12 13:14 - The Claude real E2E failure was not a broken submit hook.
  The log showed `notification_decision ... reason=selected_pane`. The test's
  “helper” pane was not visible in the default agent-only view, so the `j`
  navigation never moved selection off the Claude pane. Making the helper pane
  look like a visible Claude row fixed the suppression.
- 2026-04-12 13:15 - Lightweight failure diagnostics inside the Claude real E2E
  paid off quickly. Capturing hook payloads and the latest log on failure made
  it clear whether the problem was hook parsing, transition timing, or
  notification policy.
- 2026-04-15 13:45 - The final rerun mattered. The later setup/doctor work
  changed install behavior, local `HOME` wiring, and test expectations, so the
  real native lane needed to pass again on the final tree rather than relying
  on the earlier native proof.
