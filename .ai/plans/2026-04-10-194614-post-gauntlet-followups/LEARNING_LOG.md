---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises.
---

# Learning Log

- 2026-04-10 19:46 MST — Follow-up planning started from operator feedback that
  the release gauntlet is useful but still leaves two product-level trust gaps:
  unclear tmux-focus semantics and no explicit advertised-key contract.
- 2026-04-10 19:52 MST — The first plan version tried to trim repeated
  gauntlet execution out of `mise run verify`, but that was the wrong product
  tradeoff. The heavy lane should stay redundant on purpose so the report-
  producing release proof still runs in the final gate.
- 2026-04-10 20:17 MST — The first help-surface assertions were too tied to a
  narrow render size. The right split is: unit tests prove the stable upper
  help sections, while tmux-backed runtime and release tests prove the full
  operator copy in a wide viewport.
- 2026-04-10 20:22 MST — One startup-gauntlet assertion incorrectly assumed the
  actionable pane would still be `working` when the preview updated. Status-
  specific assertions belong only in flows that are explicitly proving status
  transitions, not in general navigation proof.
- 2026-04-10 20:37 MST — The slice closed cleanly with an explicit keybind
  matrix, clearer tmux-focus language, empty-harness skipping on `h`, and a
  new opt-in `mise run verify-native` task for real external harness drills.
- 2026-04-10 20:49 MST — `verify-ux` was also churning a tracked
  `visual-env.txt` with random temp paths on every run. The fix was to restore a
  stable placeholder after VHS capture so future heavy-validation passes stop
  generating meaningless diffs.
