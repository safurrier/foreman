# macOS Overlay Arrow Key Fix

## Goal

Fix the user-reported regression where arrow keys do not navigate/scroll in the macOS overlay after native text-editing pass-through changes.

## Hypothesis

When the search field is focused, arrow key events can arrive with private-use function-key characters. The text-editing pass-through branch treats those as normal non-control characters and returns the event to the field editor before the overlay command adapter can map Up/Down/PageUp/PageDown to navigation or preview/help scrolling.

## Scope

- Preserve native search text editing for text, selection, deletion, and left/right cursor movement.
- Keep Up/Down/PageUp/PageDown owned by the overlay outside compose mode.
- Keep compose mode arrow keys native for multiline editing.
- Extend the scripted gauntlet to prove Down Arrow changes the selected pane.
