---
id: macos-overlay-ux-checklist
title: macOS Overlay UX Checklist
description: UX review checklist for Foreman's native Swift macOS overlay.
index:
  - id: core-interaction
  - id: information-hierarchy
  - id: macos-citizenship
  - id: states
  - id: visual-capture-review
---

# macOS Overlay UX Checklist

Use this checklist with the vendored skills:

- `.agent/skills/macos-design-guidelines/SKILL.md`
- `.agent/skills/macos-app-design/SKILL.md`

The overlay is a **menu-bar utility / command-palette HUD**, not a document app.
Optimize for fast keyboard-first control of tmux agent panes.

## Core Interaction

- [ ] Global hotkey opens directly into Foreman, not a generic launcher.
- [ ] Search field is focused immediately on open.
- [ ] Overlay appears on the focused display and stays spatially stable.
- [ ] `Esc` closes reliably.
- [ ] Up/down moves selection without stealing text editing behavior.
- [ ] `Enter` focuses the selected pane.
- [ ] Compose/send is keyboard reachable.
- [ ] Refresh is keyboard reachable.
- [ ] Menu bar item exposes Open, Refresh, Settings/Help if present, and Quit.

## Information Hierarchy

- [ ] Rows show status, harness, session/window context, and workspace without
      overcrowding.
- [ ] Needs-attention/error states are visually prominent but not alarming.
- [ ] Status is not communicated by color alone.
- [ ] Detail panel separates metadata from terminal preview.
- [ ] Preview text is bounded and scrollable.
- [ ] Long paths and duplicate names remain distinguishable.

## macOS Citizenship

- [ ] Use system typography, colors, materials, and controls where possible.
- [ ] Avoid custom chrome unless it has a clear purpose.
- [ ] Standard text editing works in search/compose fields.
- [ ] App/menu commands use stable names and standard shortcuts where applicable.
- [ ] The overlay respects light/dark mode, increased contrast, reduced motion,
      and reduced transparency.

## States

- [ ] Loading state explains what is loading.
- [ ] Empty state says how to start/fix agent discovery.
- [ ] Broken `foreman` path state points to settings or launch env.
- [ ] Invalid JSON state points to control API/schema issue.
- [ ] tmux unavailable state points to `foreman --doctor`.
- [ ] Focus/send failures stay visible and actionable.

## Visual Capture Review

For each capture in `.ai/validation/macos-overlay/`, check:

- [ ] The panel is visible and centered/top-centered.
- [ ] Search focus is obvious.
- [ ] Selected row is obvious.
- [ ] Primary action is clear.
- [ ] No layout jumps, clipped labels, or unreadable contrast.
- [ ] The screenshot would make sense to a new user without terminal context.
