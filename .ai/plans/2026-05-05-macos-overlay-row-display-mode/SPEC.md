---
id: macos-overlay-row-display-mode-spec
title: Spec — macOS Overlay Row Display Mode
---

# Specification — macOS Overlay Row Display Mode

## Problem

The macOS overlay flattens Foreman's hierarchical TUI model into a single pane
list. Row titles currently use `entry.navigationTitle`, which is often the
workspace folder name. When multiple agent panes run in the same repo/vault, rows
become visually duplicated:

```text
notes-vault
Pi · notes
notes-vault
Pi · notes
```

Regular Foreman avoids this by rendering a session/window/pane hierarchy and by
using window names when windows are not elided. The overlay needs equivalent
context without reintroducing a full tree by default.

## Goals

- Keep the overlay's default unit pane-level.
- Make duplicate workspace rows distinguishable.
- Add a user setting for row title strategy.
- Keep row density compact.
- Validate with a fixture/snapshot that contains duplicate workspace names.

## Row display modes

Add `OverlayRowDisplayMode`:

- `smart` — default; use workspace when unique, otherwise window name, then pane
  title, then workspace/navigation title.
- `workspace` — always prefer workspace/navigation title.
- `session` — session name.
- `window` — window name.
- `paneTitle` — pane title, cleaned if possible.

## Smart mode behavior

For each visible entry:

1. Count duplicate `workspaceName`/`navigationTitle` values in the current visible
   entries.
2. If the workspace/navigation title is unique, use it.
3. If duplicated, prefer a non-generic `windowName`.
4. If `windowName` is generic or empty, prefer `title` when not empty/pane id.
5. Fallback to `navigationTitle`.

Secondary row text should include enough context to reconstruct identity:

```text
<harness> · <session> · <workspace/window/pane>
```

At minimum, for duplicated titles, include workspace and pane id in the subtitle.

## Settings

Add a Settings → View picker:

```text
Row title: Smart / Workspace / Session / Window / Pane Title
```

Persist through `OverlayPreferences`.

## Validation

- Add Swift unit tests for duplicate workspace row titles.
- Add or update a fixture with duplicate `notes-vault` entries.
- Add a headless snapshot state showing duplicates disambiguated.
- OCR should assert window names like `project-alpha review` or `project-alpha skill` appear.
- `mise run verify-macos-overlay` passes.

## Acceptance

For multiple notes app panes in one `notes` session, the overlay should show
rows resembling:

```text
project-alpha startup
Pi · notes · notes-vault · %35

project-alpha review
Pi · notes · notes-vault · %33
```

rather than repeated bare `notes-vault` titles.
