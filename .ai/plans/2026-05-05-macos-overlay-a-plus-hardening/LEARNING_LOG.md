# Learning Log

## 2026-05-05

- Started A+ hardening loop from code review feedback.
- Prioritized hotkey reliability first because `Ctrl+F` still does not toggle for the user and current status can be misleading: KeyboardShortcuts can mark a shortcut registered even when Carbon registration failed internally.
- Slice 1 switched the default/recorder shortcut path to Foreman's own Carbon `HotkeyController`, so Ctrl+F no longer depends on KeyboardShortcuts' opaque legacy handler registration. The recorder still owns persistence/UI.
- Slice 2 kept command-palette global typing outside text fields but now lets AppKit handle ordinary text editing whenever SwiftUI's search/compose fields are using the field editor. This preserves Cmd+A, Option+Delete, cursor movement, and selected-text replacement semantics.
- Slice 3 moved process output collection from blocking read threads to readability handlers, added hard timeout cleanup, and made bare executable lookup intentional by routing through `/usr/bin/env`.
- Slice 4 made reload a replaceable task with a monotonically increasing generation id. Older responses can finish, but they no longer overwrite newer state.
- Slice 5 changed `selectedEntry` from an action-only fallback into normalized state. Query/filter changes now update `selectionId` to a visible row instead of letting actions target a row that the list no longer highlights.
- Slice 6 was intentionally mechanical: no API changes, just split the large Core module by ownership so future diffs have better locality.
- Slice 7 made the reviewer's validation recommendation concrete with `mise run validate-macos-overlay-change`, then documented that this lane is required for overlay behavior changes while plain Swift tests remain the fast loop.
- Slice 8 kept the performance change local: row title/subtitle presentation is now batch-produced from visible entries, avoiding duplicate workspace rescans per row. Accessibility labels now include title, status, identity context, and PR presence without depending only on color.
