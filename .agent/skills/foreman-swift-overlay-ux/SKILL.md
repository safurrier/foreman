---
name: foreman-swift-overlay-ux
description: Use when designing, reviewing, or changing Foreman's native Swift macOS overlay UI. Applies macOS HIG guidance, Foreman-specific command-palette UX, screenshot review, accessibility, and visual iteration rules.
---

# Foreman Swift Overlay UX

Use this skill when working in `apps/macos-overlay/`, reviewing screenshots in
`.ai/validation/macos-overlay/`, changing keyboard behavior, or polishing the
native macOS overlay.

## Load First

Read these files before making UX changes:

1. `docs/macos-overlay/ux-checklist.md`
2. `.agent/skills/macos-design-guidelines/SKILL.md`
3. `.agent/skills/macos-app-design/SKILL.md`
4. `.agent/skills/macos-app-design/references/macos-design-guide.md`

## Product Frame

The overlay is a **menu-bar utility / command-palette HUD** for Foreman agents.
It should feel closer to Spotlight/Raycast directness than a generic document
window.

Target flow:

```text
global hotkey → search focused → select agent → focus/send → close
```

## Rules

- Search must be focused immediately on open.
- Keyboard-only operation is required for the happy path.
- Preserve Mac text editing behavior in search and compose fields.
- Use system typography, colors, materials, and controls before inventing custom
  chrome.
- Make status visible through text/shape as well as color.
- Separate agent metadata from terminal preview.
- Bound preview text and make it scrollable.
- Loading, empty, error, invalid JSON, tmux unavailable, and broken foreman path
  states must each explain recovery.
- Do not add visual polish without updating or reviewing screenshot artifacts.

## Screenshot Review Loop

Run:

```bash
./scripts/capture-macos-overlay.sh
open .ai/validation/macos-overlay/attention.png
```

Then check:

- panel position and size
- search focus visibility
- selected row visibility
- row density and truncation
- detail hierarchy
- contrast in current appearance
- whether a new user can infer the primary action

Record conclusions in the active plan's `VALIDATION.md` or `LEARNING_LOG.md`.
