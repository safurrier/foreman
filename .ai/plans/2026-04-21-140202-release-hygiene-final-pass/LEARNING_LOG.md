# Learning Log — release-hygiene-final-pass

## 2026-04-21

### Starting point

- Actionable-pane semantics are inconsistent between input/focus actions and repo-scoped behaviors.
- Validation evidence still depends on dated plan directories.
- Capture failures are counted globally but not attributed to panes or shown clearly in the preview panel.
- No stray benchmark-name matches were found at slice start, but the repo still needs a final scrub during closeout.

### What matched the plan

- Splitting actionable workspace context from aggregate row metadata made the notification, PR, and diagnostics fixes line up cleanly.
- Stable validation roots under `.ai/validation/{ux,release}` were enough for CI and release consumers once the tasks themselves wrote there and failed on missing outputs.
- Preview provenance only needed a small explicit model (`captured`, `reused`, `failed`) to make degraded capture visible without destabilizing the runtime.

### What diverged

- Two runtime-style tests had to be updated after the actionable-pane fix because they were previously proving notification behavior for the currently selected actionable pane.
- The first heavy-lane rerun exposed a real `verify-ux` regression: I had moved artifact outputs to the stable root but left the generated VHS tape on invalid syntax for the installed VHS parser.
- The release gauntlet also needed timing adjustments after the off-selection alpha notification proof stopped using selected-pane UI waits.

### Fixes and lessons

- For container-row semantics, keep aggregate row identity separate from actionable pane context. Reusing one cached workspace field for both is what causes notification and PR mismatches.
- When a workflow task generates artifacts, migrate the generator inputs too. Moving only the output directory is not enough.
- In tmux dashboard tests, if the product semantics intentionally suppress notifications for the selected actionable pane, the test should select a different row or transition a different pane explicitly instead of relying on startup ordering.
- For off-selection transition tests, refresh-count waits plus a short settle are more durable than waiting on selected-pane preview text that is no longer supposed to change.

### What would make this more one-shot next time

- Add a narrow task-level smoke for `verify-ux` any time the tape generator changes, before rerunning the whole heavy lane.
- Keep one helper for “select a non-actionable target and prove off-selection notification behavior” so runtime and release smokes do not each rediscover the same pattern.
