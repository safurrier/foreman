# Learning Log

## 2026-04-24

The sort mode already had a clean enum and label surface. The important implementation detail is that `AppState::with_inventory` rebuilds visible rows immediately using `SortMode::default()`, so runtime config must be applied through a helper that also calls `rebuild_visible_state()`.

Config spelling is `attention-recent` to avoid putting the UI arrow glyph into config. The UI still renders `attention->recent`.

## 2026-04-25 Scope Expansion

Expanded the slice to include UI preference persistence and small UX polish items requested by the user. Key risk is keeping persistence out of pure reducer/render paths; runtime action handling is the right seam for disk writes because reducer remains pure and effects already centralize side effects.

## 2026-04-25 Implementation Notes

Added a small `ui_preferences` service so runtime can persist preferences after operator actions without putting I/O in the reducer or renderer. Cache controls now live under `[monitoring]`, while default sort/theme stay under `[ui]`. Notification polish stayed to context-rich per-agent messages rather than grouped summaries, because grouping would require a separate notification queue and timing policy.


## 2026-04-25 Validation Notes

Full `mise run check` exposed a real persistence isolation bug: storing `ui-state.json` under the log directory parent made separate temp log directories share one state file in tests, and could make user overrides bleed across unrelated config roots. The durable fix is to key UI preferences by the resolved config file directory instead.
