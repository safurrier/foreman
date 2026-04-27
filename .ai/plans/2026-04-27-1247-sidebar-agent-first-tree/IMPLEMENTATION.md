# Implementation Plan

1. Add a helper for default agent-first singleton window elision.
2. Apply it consistently to visible targets and rendered entries.
3. Adjust pane rendering so the direct child row reads as semantic agent content.
4. Update SPEC and tests for default semantic tree vs topology modes.
5. Run focused tests, `mise run check`, and `mise run verify-ux`.
6. Push the PR update and poll CI/reviews.
