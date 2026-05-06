---
id: macos-overlay-settings-filters-parity-investigation
title: Investigation — macOS Overlay Settings and Filters Parity
---

# Investigation Notes

## User observations

- Wants settings for default popup size.
- Wants settings parity with Foreman concepts like notifications.
- Search bar has awkward empty vertical space above cursor/text.
- `Tab` does not feel like it selects the Details/PR pane when a PR is present.
- Wants parity for view filters such as specific agents and recent/attention
  ordering.

## Current implementation snapshot

- Settings UI lives in `apps/macos-overlay/Sources/ForemanOverlay/main.swift` as
  `SettingsView` and currently only exposes:
  - shortcut recorder
  - reset shortcut button
- Panel size is hardcoded in `OverlayPanelController`:
  - width: 820
  - height: 560
- Panel is not currently user-resizable.
- `ForemanClient` has `includeAllPanes`, but it is only wired through env:
  - `FOREMAN_OVERLAY_ALL_PANES=1`
- Store filtering currently only supports free-text query.
- Sort is API order only.
- Help/footer mention details region, but PR card is a `Link` rendered inside
  detail content and not exposed as a keyboard action.

## Likely root cause of search bar visual issue

`OverlayView.searchBar` uses a large font and broad vertical padding:

```swift
.font(.system(size: 22, weight: .medium))
.padding(16)
```

In the taller panel state, the search row appears visually too tall, and the
field/cursor sits low relative to the empty space. A custom search component with
fixed row height and aligned text should fix it.

## Parity mapping

| Foreman TUI concept | macOS overlay current | Proposed |
|---|---|---|
| Search | type-to-search | keep |
| Flash | Cmd+J | keep |
| All panes | env only | Settings toggle |
| Harness filter | missing | Settings picker / footer chip |
| Sort modes | missing | stable / attention / recent picker |
| Status filters | missing | Settings picker |
| Notifications | runtime/TUI docs | Settings placeholder until control API exists |
| PR actions | clickable Link only | detail region keyboard action(s) |
| Theme | Cmd+T only | maybe Settings later |
| Panel size | hardcoded | persisted Settings |

## Suggested implementation order

1. Preferences model + persistence.
2. Popup size setting.
3. View filters/sort.
4. Search bar visual component.
5. Detail/PR keyboard actions.
6. Notifications placeholder/parity note.
