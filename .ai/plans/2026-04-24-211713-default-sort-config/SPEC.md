---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — default-sort-config

## Problem

Operators can choose useful UI modes at runtime, but Foreman forgets them across popup launches. Popup startup also has limited cache controls and some status surfaces still feel sparse: notifications lack enough location context, PR/cache states do not explain what is happening, and reopening a popup does not return to the last relevant target.

## Requirements

### MUST

- Support `[ui].default_sort = "stable"` and `[ui].default_sort = "attention-recent"`.
- Keep `stable` as the default when config is missing.
- Apply configured/default/persisted sort before first render for cached popup and normal live startup.
- Persist user UI choices after runtime changes: sort mode, theme, harness filter, non-agent visibility toggles, collapsed sessions, and selected target when possible.
- Restore the last selected target on startup when it still exists, falling back safely when it does not.
- Add config control for popup startup cache freshness.
- Surface cache age/path details in startup diagnostics/readout.
- Improve notification text with clearer session/window/pane context and source/provenance.
- Improve PR panel feedback for loading/unavailable/manual refresh states.
- Preserve existing keyboard behavior and no-color-safe rendering.

### SHOULD

- Accept `attention-first` as a compatibility alias because the internal enum name historically used that wording.
- Log resolved default sort and persisted preference path during bootstrap.
- Keep persistence failures non-fatal and visible via diagnostics/logs rather than breaking the dashboard.

## Constraints

- Do not add broad new async work to render/reducer paths.
- Do not change sort comparator semantics.
- Do not make notifications noisier by adding new transition types.
- Keep the slice cohesive; defer any large notification grouping system if it needs a separate queue.
