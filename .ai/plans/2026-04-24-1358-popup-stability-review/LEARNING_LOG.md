# Learning Log

## 2026-04-24 13:58

Started from user feedback after jumping directly into implementation. Corrected course by creating this plan bundle before continuing.

Findings so far:

- Startup cache persists on disk under the configured cache directory, keyed by tmux socket identity, but was treated as stale after only 15 seconds.
- Default sort was labeled `recent->status` and used volatile activity/status signals, causing visible row movement during normal monitoring.
- Notification policy suppressed the selected pane globally; popup mode makes that suppression surprising because the selected pane is often the one the user wants updates about.
- `mise run check` initially failed in `runtime_uses_configured_notification_backend_order` because the test expected old volatile ordering after stable sort changed row positions.

## 2026-04-24 14:03

Adapted runtime notification test after stable sort changed visual row positions. The test now selects `alpha` via search rather than assuming a fixed number of `j` presses, which better matches the UX contract and avoids coupling to row ordering details.

## 2026-04-24 14:10

Runtime popup validation confirmed a review finding: search filtered to `beta`, but committing search kept selection on `alpha` because the reducer used the stale pre-filter selection. Fixed search commit to select the active visible match and fixed flash jump-and-focus to resolve session/window targets through the same actionable pane path as normal focus.

## 2026-04-24 14:18

`mise run check` passes after formatting. Slice expanded from popup cache/sort/notifications to include search and flash target resolution because runtime validation proved that bug directly affected popup focus behavior.
