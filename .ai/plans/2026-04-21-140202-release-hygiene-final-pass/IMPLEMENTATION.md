# Implementation — release-hygiene-final-pass

## Approach

Treat this as one release-closeout slice with four tracks:

1. split aggregate row metadata from actionable pane context
2. move validation artifacts to a stable root and wire CI/release to it
3. add per-pane preview provenance and pane-attributed capture failure logging
4. scrub any lingering benchmark-name references

The state/UI changes stay narrow by extending the cached visible-row model instead of teaching every caller how to resolve panes again.

## Steps

1. Extend the visible-row cache so session/window rows carry both aggregate workspace summary and actionable pane workspace context.
2. Update notification suppression, PR lookup, and runtime diagnostics to use actionable pane semantics consistently.
3. Add direct reducer/state/runtime regressions for session-row and window-row cases.
4. Introduce a stable validation evidence root and update `verify-ux`, `verify-release`, CI, release, and docs.
5. Extend the pane model with preview provenance and capture failure detail.
6. Surface preview provenance in logs and the preview panel without breaking soft-failure behavior.
7. Search for and remove stray textual benchmark-name references.
8. Run focused tests first, then the full validation ladder, then close out the plan.
