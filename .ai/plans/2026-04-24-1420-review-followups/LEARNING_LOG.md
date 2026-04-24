# Learning Log

## 2026-04-24 14:20

Created follow-up slice after completing popup stability work. Prioritized user-data-loss paths first: compose draft, rename modal, and spawn modal failure handling.

## 2026-04-24

Effect execution was one-way: failed tmux send/rename/spawn only raised alerts after reducer had already cleared the draft/modal. Added explicit restore actions invoked by runtime error branches.

## 2026-04-24 PR responsiveness

PR lookup was debounced but still executed synchronously in the event loop, so slow `gh pr view` could block rendering and input. Moved lookup execution to a background worker and kept result application on the main reducer path.

## 2026-04-24 native/subprocess hardening

Native signal files are pruned against live pane IDs before overlays apply, reducing stale file reuse after panes close. Pull-request subprocess helpers now use bounded waits so slow `git`, `gh`, browser, or clipboard commands cannot hang indefinitely.

## 2026-04-24 preview scroll

Preview mode previously existed as state but did not own any scrolling behavior. Added a `preview_scroll` offset, preview-focused key mappings, reducer-owned scroll actions, render scroll application, and footer hints.

## 2026-04-24 completion

Full `mise run check` passes. This follow-up slice addressed the remaining review items that were safe to fix without a broader rewrite: draft preservation, async PR lookup, stale native signal pruning, bounded subprocess waits, and preview scroll mode.

## 2026-04-24 PR Review Loop

Codex review found that deleting native signal files based on one tmux inventory could interfere with agents from another tmux server sharing the same native signal directory. Removed destructive pruning; stale signal files are already ignored unless their pane id appears in the current inventory.

Preview End used `u16::MAX` as a sentinel, so render now clamps the scroll offset against the current details text height before drawing.

## 2026-04-24 External PR Comments

Codex GitHub review caught that preview-focused scroll mode accidentally narrowed normal shortcut availability. Kept scroll-specific `j/k/PageUp/PageDown/Home/End` behavior while allowing normal operator shortcuts in preview focus.

Search commit now resets preview scroll so selecting a result cannot inherit an irrelevant deep details offset from the prior pane.
