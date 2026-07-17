# Foreman macOS App

**When a user corrects you or provides you with tribal knowledge/gotchas —
something you could not have known from reading the code or your prompt — you
MUST document it in an AGENTS.md file before continuing.** Write the correction
in the AGENTS.md closest to where the issue occurred.

This Swift package is Foreman's native macOS control surface over the Rust JSON control API. Read `docs/macos-overlay/architecture.md` for ownership seams and `docs/macos-overlay/validation.md` before choosing proof for a change.

## Commands

From the repository root, run `mise run validate-macos-overlay-change` for overlay behavior, focus, keyboard, bundle, or visual changes.

## Gotchas

- **DO** send view intents through `OverlayStore`, typed control requests through `ForemanClient`, and subprocess work through `ProcessRunner`. **NOT** let views or the store call `ProcessRunner` directly or discover tmux state in Swift. **BECAUSE** the client/process seam keeps Rust as the source of tmux and harness truth.

- **DO** key rows, selection, and card merges by `sourcePaneId`, and pass `sourceId` on remote actions. **NOT** treat `paneId` as globally unique. **BECAUSE** local and remote sources can expose the same tmux pane id.

- **DO** let AppKit text views handle ordinary search and compose editing, while routing overlay navigation through the keyboard adapter and store. **NOT** manually implement cursor movement, selection replacement, or command editing in `OverlayStore`. **BECAUSE** the app must preserve native Mac text-field behavior without losing command-palette navigation.

- **DO** keep app lifecycle, panels, menus, shortcuts, and application activation in the AppKit shell; keep view state and intents in Core/SwiftUI. **NOT** make a view own global monitors or terminal activation. **BECAUSE** window focus and app activation require AppKit lifecycle ownership and test seams.

- **DO** cancel or generation-guard asynchronous reload and selected-card work before applying results. **NOT** let an older request overwrite a newer selection or inventory. **BECAUSE** remote and extension requests may finish out of order.

- **DO** use the `KeyboardShortcuts` package for persisted shortcut recording and global shortcut handling. **NOT** register a second custom Carbon hotkey for the persisted shortcut. **BECAUSE** mixed ownership races the recorder and can leave Settings showing stale registration failures.

- **DO** use the required overlay lane in `docs/macos-overlay/validation.md` for behavior, focus, keyboard, bundle, or visual changes. **NOT** treat the inner-loop unit command as final proof for those surfaces. **BECAUSE** Swift unit tests do not exercise fake-Foreman UI events, real tmux, snapshots, OCR, or app-bundle launch.

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-07-15 -->
