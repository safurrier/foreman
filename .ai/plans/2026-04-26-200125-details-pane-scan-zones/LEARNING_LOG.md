---
id: plan-learning-log
title: Learning Log
description: Dev diary for this unit of work.
---

# Learning Log

## 2026-04-26

- External TUI/dashboard scan: dense operator views work best with stable regions, aligned key/value rows, semantic color, and state/context/action ordering.
- Prior Foreman plans warn that Details improvements can easily blow the 100x32 vertical budget; this pass should improve structure mostly by changing row shape, not adding many rows.
## Implementation notes

- Kept the pass render-only: no reducer, runtime, or service changes.
- Reordered Pull request above Selected target so PR state has a stable early scan zone when present.
- Aligned recurring labels with semantic value styles; used a compact event row for long notification copy to avoid wrapping too early.
## Self-review fixes

- Review caught that the longest aligned label had no value gap. Fixed the helper instead of patching individual rows.
- Review also caught that broad glyph waits could pass from sidebar text. Runtime and release waits now target Details rows directly.
