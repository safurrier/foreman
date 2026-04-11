---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — focus-hints-artifacts

1. Add focus-aware operator hint helpers.
   - Extend `AppState` with small helper methods that describe the current
     focus, the selected actionable pane, and native vs compatibility source
     language.
   - Keep this logic out of the renderer so tests can assert the contract more
     directly.

2. Rebuild the footer and help around those helpers.
   - Make normal-mode footer lines depend on `Focus` and selection shape.
   - Add a “right now” section near the top of help.
   - Add a source legend that explains native vs compatibility in plain
     language.

3. Surface provenance in preview details.
   - Selected panes should show the status source and confidence language.
   - Session and window selections should show the actionable target pane’s
     provenance where that helps explain what `Enter`, `f`, and `i` will use.

4. Strengthen proof.
   - Add render tests for focus-aware footer variants and provenance copy.
   - Extend the runtime smoke and release gauntlet to assert the new help and
     preview content in the real compiled binary.

5. Surface proof in CI and sync docs.
   - Upload the existing UX artifact directory and release-gauntlet artifact
     directory from GitHub Actions.
   - Update the spec, workflow docs, and validation log.
