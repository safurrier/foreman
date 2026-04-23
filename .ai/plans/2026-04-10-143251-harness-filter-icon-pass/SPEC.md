---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — harness-filter-icon-pass

## Problem

Foreman's dashboard is functionally broad but still harder to read than it should
be. The sidebar remains too text-heavy, the current harness badges (`CLD`, `CDX`,
etc.) are opaque without prior knowledge, and browse-vs-act affordances are still
weaker than they should be for a session/window/pane tree.

This pass focuses on making the TUI easier to scan and operate:
- keep the default agents-only view
- add harness/provider-level filtering so operators can isolate one agent family
- replace opaque harness text badges with more visual, terminal-safe marks
- strengthen the in-product legend/help so the UI teaches itself
- add live validation that exercises the new view/filter flow in the real binary

## Requirements

### MUST

- Keep the default runtime behavior agents-only: non-agent panes remain hidden
  until explicitly toggled on.
- Add a harness/provider filter that can be cycled from the keyboard and applies
  consistently across sessions, windows, and panes.
- Preserve selection and actionable-pane behavior when filters change.
- Replace sidebar harness labels with compact visual marks that are easier to scan
  than `CLD`/`CDX` while remaining terminal-safe.
- Provide a clear legend/help surface that explains harness marks, status marks,
  and filter controls.
- Improve browse-vs-act clarity for session and window rows so the operator can
  tell what `Enter`, `f`, and `i` will target.
- Add or update tests across reducer, render, and live tmux runtime layers.
- Fold the new UX checks into the existing heavy UX validation path.

### SHOULD

- Keep the footer shorter and more role-based than raw command-dump prose.
- Surface the active harness filter prominently in the dashboard.
- Use Unicode-first visuals with ASCII-safe fallback rather than color-only cues.
- Refresh UX artifacts so the visual changes are captured alongside the code.

## Constraints

- Do not depend on true raster images or terminal-specific logo rendering. The app
  must remain portable across normal Ratatui + Crossterm + tmux environments.
- Do not regress no-color or low-capability terminals.
- Avoid adding new blocking work to navigation paths; the lag fix from the prior
  pass must remain intact.
- Preserve the existing command/action/reducer/effects shape and keep rendering
  pure.
