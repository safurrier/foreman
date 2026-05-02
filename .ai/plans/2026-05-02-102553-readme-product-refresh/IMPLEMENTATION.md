---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — readme-product-refresh

## Approach

Split the material by reader intent:

- README: product positioning, quickstart, native integration summary, and links.
- Operator guide: setup, doctor, dashboard controls, preferences,
  notifications, native hook wiring, and support matrix.
- Docs index: route operators and contributors to the right durable page.
- Demo source: commit a VHS tape that can render a short quickstart demo later.

## Steps

1. Audit current README sections and docs routing.
2. Create `docs/operator-guide.md` for moved operator reference material.
3. Rewrite `README.md` as the project front door.
4. Add a `demos/readme-quickstart.tape` source artifact.
5. Update docs routing.
6. Run docs verification and the repo check gate.
