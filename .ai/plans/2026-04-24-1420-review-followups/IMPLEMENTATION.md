# Implementation Plan

1. Inspect effect/result flow for send, rename, and spawn failures.
2. Preserve compose draft/modal state on failed side effects.
3. Add reducer/runtime tests for failure preservation.
4. Inspect PR lookup execution path and patch obvious blocking behavior if feasible.
5. Inspect native signal freshness and subprocess timeout seams.
6. Run focused tests and `mise run check`.
