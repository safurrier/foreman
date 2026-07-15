# App State Core

**When a user corrects you or provides you with tribal knowledge/gotchas —
something you could not have known from reading the code or your prompt — you
MUST document it in an AGENTS.md file before continuing.** Write the correction
in the AGENTS.md closest to where the issue occurred.

This module owns UI-facing state, key-to-command mapping, command-to-action mapping, and the pure reducer that emits runtime effects. Product behavior remains canonical in `SPEC.md`; cross-module boundaries remain canonical in `docs/architecture.md`.

## Commands

From the repository root, run `cargo test --lib app::` for this module's unit tests.

## Gotchas

- **DO** route input through `Command`, then `Action`, then `reduce`. **NOT** match raw key events in reducer or runtime business logic. **BECAUSE** mode precedence and input behavior stay testable only when key mapping is separate from state transitions.

- **DO** keep drafts, modal targets, search/flash restore targets, focus, and selection in `AppState`. **NOT** move interaction state into widgets or runtime-local variables. **BECAUSE** cancellation, refresh reconciliation, and rendering depend on one reducer-owned state model.

- **DO** preserve source-scoped keys such as `PaneKey` in selection, caches, and effects. **NOT** use a bare tmux pane id as global identity. **BECAUSE** different sources may expose the same tmux pane id.

- **DO** reconcile selection and active interaction state whenever inventory or visibility changes. **NOT** update `inventory`, filters, sorting, or collapsed state without the existing reconciliation paths. **BECAUSE** stale targets can send effects to the wrong pane or leave the UI pointing at no visible row.

- **DO** return explicit `Effect` values for I/O. **NOT** call tmux, GitHub, notification, clipboard, browser, or filesystem APIs from this module. **BECAUSE** the reducer must remain deterministic and rendering must remain pure.

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-07-15 -->
