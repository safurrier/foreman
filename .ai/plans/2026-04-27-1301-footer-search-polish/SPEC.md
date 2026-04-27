# SPEC — Footer/Search Polish

## Problem

The bottom info bar should explain glyph/status meaning more clearly, and search should feel inline and modal-light like vim/neovim rather than appearing as a floating popup.

## Requirements

- Search mode shows the active `/<query>` and match count in the footer/info bar.
- Search mode does not render a floating overlay over the dashboard.
- Footer hints use labeled groups that are easier to scan.
- Help/legend explains that pane rows use harness glyphs plus status color/styling.
- Default direct pane labels prefer the elided window title when it is more useful than a duplicate pane/workdir title.
- Validation covers render, runtime, check, and UX artifacts.
