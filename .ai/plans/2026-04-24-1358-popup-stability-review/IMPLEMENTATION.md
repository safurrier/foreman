# Implementation Plan

## Slice 1: Popup Stability

1. Inspect startup cache read/write path and default cache location.
2. Increase popup cache freshness window while keeping stale protection.
3. Change default sort mode to stable ordering.
4. Keep attention-first sorting as an explicit operator choice.
5. Adjust popup notification policy so selected rows are not suppressed in popup mode.
6. Update README, SPEC, and render help labels.
7. Add tests for stable sort and popup notification behavior.
8. Run focused tests, then `mise run check`.

## Follow-Up Queue From Review

These should be separate slices unless the user asks to bundle them:

1. Search Enter should handle selected session/window matches, not only pane matches.
2. Flash `S` should focus session/window targets instead of only selecting them.
3. Failed send should preserve compose draft instead of losing user input.
4. Failed rename/spawn should preserve modal content on error.
5. PR lookup should avoid blocking the UI thread.
6. Native signal reads should guard better against stale files.
7. Preview scroll mode is specified but not fully implemented.
8. Subprocess calls should have clearer timeouts and cancellation behavior.
