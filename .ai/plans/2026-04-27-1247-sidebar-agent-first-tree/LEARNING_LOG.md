# Learning Log

## Initial Decision

The earlier flattening was directionally right for default agent-first mode, but the popup compact layout made it feel like stacked horizontal sections. With popup consistency fixed, `session -> agent` is the cleaner default.


## Implementation Notes

- The default sidebar now elides singleton window rows again, but popup mode keeps a side-by-side layout so the result reads as a tree rather than stacked bands.
- Pane rows use the harness glyph plus status-colored text instead of a separate bullet-like status marker.


## Validation Notes

- `mise run check` caught a stale runtime assertion for the removed bullet marker; updating that assertion confirmed the intended `✦ foreman` row.
- `verify-ux` now captures the cleaned default and popup sidebar presentation.
