# Review Followups

## Problem

The intensive review surfaced several correctness and UX issues beyond popup cache/sort/notification behavior.

## Goals

- Preserve user-authored text when runtime effects fail.
- Avoid modal data loss for rename/spawn failures.
- Improve responsiveness where synchronous background lookups can block input/rendering.
- Harden native signal and subprocess edge cases where small failures cause confusing UI behavior.
- Keep each fix minimal, covered by tests, and aligned with `SPEC.md`.

## Non-Goals

- Do not rewrite the runtime architecture.
- Do not change native hook protocols unless a bug requires it.
- Do not add new dependencies without clear need.
