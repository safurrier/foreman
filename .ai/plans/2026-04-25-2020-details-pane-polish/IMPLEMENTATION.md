# Implementation — Details pane polish

1. Audit `src/ui/render.rs` Details composition and existing buffer tests.
2. Add a compact selected-target summary section with status, source, workspace, and pane identity.
3. Reduce duplicate action prose by moving common affordances into concise `Next` lines.
4. Add render regression coverage for summary and key actions.
5. Run focused render tests, runtime dashboard, and `mise run check`.
