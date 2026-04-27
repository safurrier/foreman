# Learning Log

## Initial Notes

- Search should use the existing footer/info bar rather than a modal overlay.
- Agent-first elision should not discard a meaningful window title when pane/workdir names duplicate.


## Implementation Notes

- Search now lives in the footer as `/<query>` with match count, keeping the dashboard visible while filtering.
- Footer action hints use labeled groups (`Move:`, `Use:`, `Find:`, `Sort:`) for faster scanning.
- Elided singleton window titles are used as direct pane labels, so meaningful task/window names are not lost when workspace names duplicate.


## Validation Notes

- Full check caught stale help/footer/release expectations; updating them verified the labeled footer contract.
- The search UX artifact now captures inline footer search instead of a floating query modal.
