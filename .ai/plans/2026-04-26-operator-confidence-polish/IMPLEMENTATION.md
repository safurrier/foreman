# Implementation — Operator confidence polish

1. Diagnose live Claude Code error state and capture root cause in the learning log.
2. Make Claude compatibility status prefer recent preview lines over stale scrollback.
3. Add main-pane confidence blocks: activity digest, recent events, empty/degraded/first-run guidance.
4. Add config readout utility using existing CLI flag style.
5. Improve doctor next-step copy with explicit setup/config/cache commands.
6. Map Esc to quit in normal mode only; preserve Esc behavior in overlays/modals/input.
7. Validate with focused tests, runtime dashboard, and `mise run check`.
