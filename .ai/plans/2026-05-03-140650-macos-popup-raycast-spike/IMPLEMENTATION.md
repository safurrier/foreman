---
id: macos-popup-raycast-spike-implementation
title: macOS Popup + Raycast Spike Implementation Plan
description: >
  Chunked implementation plan for adding Foreman JSON/control commands and a
  Raycast extension spike.
---

# Implementation — macos-popup-raycast-spike

## Recommended Shape

Treat this as two loosely-coupled frontends over the same tmux/domain seams:

1. **Foreman control API**: new one-shot CLI subcommands implemented in Rust,
   tested with existing fake tmux adapters and real-tmux smoke tests.
2. **Raycast extension**: TypeScript UI that shells out to the installed
   `foreman` binary using argument arrays, parses JSON, and invokes focus/send
   commands.

Do not add Raycast or macOS GUI dependencies to the main Ratatui runtime.

## Proposed CLI Surface

```bash
foreman agents --json
foreman agents --json --all-panes
foreman focus --pane %42
foreman send --pane %42 --text "please continue"
printf 'multi\nline' | foreman send --pane %42 --stdin
```

Potential future terminal activation:

```bash
foreman open --pane %42 --terminal-app WezTerm
```

But for the first Raycast spike, prefer Raycast-side activation so Foreman stays
platform-light.

## JSON Shape Draft

```json
{
  "schemaVersion": 1,
  "generatedAt": "2026-05-03T14:06:50Z",
  "inventory": {
    "totalSessions": 3,
    "totalWindows": 5,
    "totalPanes": 9,
    "visibleSessions": 2,
    "visiblePanes": 4
  },
  "entries": [
    {
      "id": "pane:%42",
      "kind": "pane",
      "paneId": "%42",
      "sessionId": "$1",
      "sessionName": "foreman",
      "windowId": "@3",
      "windowName": "agent",
      "title": "Claude Code — foreman",
      "navigationTitle": "agent",
      "harness": "claude-code",
      "status": "needs-attention",
      "statusSource": "native-hook",
      "isAgent": true,
      "currentCommand": "claude",
      "workingDir": "/workspace/foreman",
      "preview": "recent pane output..."
    }
  ],
  "diagnostics": []
}
```

Keep this as a public DTO. Do not serialize raw `AppState` as the external API;
that would couple Raycast to internal reducer/rendering state.

## Chunk 1 — Add command parsing without behavior regression

**Goal:** Introduce subcommands while preserving existing top-level utility flags
and default interactive startup.

**Acceptance Criteria**

- [ ] `foreman` still launches the interactive dashboard.
- [ ] Existing flags (`--doctor`, `--setup`, `--config-show`, `--popup`) still
      parse and behave as before.
- [ ] New subcommands parse: `agents`, `focus`, `send`.

**Tests**

- CLI parser unit tests for plain startup, existing flags, and new subcommands.
- Help snapshot or targeted help assertions for the new commands.

**Files**

- `src/cli.rs` — add `Commands` enum and dispatch.
- `SPEC.md`, `README.md`, `docs/operator-guide.md` — later docs once behavior
  is implemented.

## Chunk 2 — Build public agent DTOs

**Goal:** Convert `AppState`/visible target entries into stable serializable data.

**Acceptance Criteria**

- [ ] DTOs have explicit `Serialize` derives and documented `schemaVersion`.
- [ ] Entries are flat and palette-friendly.
- [ ] Session/window rows can be omitted initially; pane rows are enough for
      Raycast MVP.
- [ ] No renderer-specific strings are required for consumers.

**Tests**

- Unit test with fixture inventory verifies fields for Claude/Codex/Pi/non-agent
  panes.
- Unit test verifies default agent-only filtering.

**Files**

- New `src/services/control_api.rs` or `src/cli/control.rs`.
- `src/app/state.rs` may expose small read-only helpers if needed.

## Chunk 3 — Implement `foreman agents --json`

**Goal:** One-shot inventory command for external clients.

**Acceptance Criteria**

- [ ] Prints pretty or compact JSON to stdout.
- [ ] Emits human/log diagnostics to stderr only.
- [ ] Handles missing tmux with a structured error and non-zero exit.
- [ ] Supports hidden/test `--tmux-socket` or existing runtime socket override for
      real-tmux tests.

**Tests**

- Fake-backed DTO tests.
- Real tmux smoke that starts one supported harness-looking pane and asserts JSON
  includes its pane id and status.

**Files**

- `src/cli.rs`
- `tests/tmux_inventory.rs` or new `tests/control_api.rs`

## Chunk 4 — Implement `focus` and `send`

**Goal:** External action commands use existing tmux adapter semantics.

**Acceptance Criteria**

- [ ] `focus --pane` delegates to `TmuxAdapter::focus_pane`.
- [ ] `send --pane --text` and `send --pane --stdin` delegate to
      `TmuxAdapter::send_input`.
- [ ] Success output is brief for humans and JSON-capable for scripts if needed.
- [ ] Errors are actionable and non-zero.

**Tests**

- Fake backend verifies delegation.
- Real tmux smoke verifies active pane changes and sent text appears.

**Files**

- `src/cli.rs`
- `src/adapters/tmux.rs` if a small public helper is needed.
- `tests/tmux_focus.rs`, `tests/runtime_dashboard.rs`, or new
  `tests/control_actions.rs`.

## Chunk 5 — Raycast extension skeleton

**Goal:** Create a dev/private Raycast extension that renders `foreman agents`.

**Acceptance Criteria**

- [ ] Extension can be developed with `npm install && npm run dev`.
- [ ] Preference config supports `foremanPath`.
- [ ] Main command uses `useExec(foremanPath, ["agents", "--json"], ...)`.
- [ ] JSON parse failures show a Raycast toast with stderr/context.

**Files**

- `integrations/raycast/foreman/package.json`
- `integrations/raycast/foreman/src/index.tsx`
- `integrations/raycast/foreman/src/foreman.ts`
- `integrations/raycast/foreman/README.md`

## Chunk 6 — Raycast list/detail/actions

**Goal:** Make the extension useful enough to validate the workflow.

**Acceptance Criteria**

- [ ] List rows show agent title, session/window, status, harness, and workspace.
- [ ] Detail panel shows preview and metadata.
- [ ] Primary action runs `foreman focus --pane <id>` and closes/suspends Raycast
      if appropriate.
- [ ] Secondary action opens a form or uses selected text to send input.
- [ ] Refresh action calls `revalidate()`.

**Tests / Validation**

- TypeScript typecheck/lint if Raycast template provides it.
- Manual dev validation through Raycast with a live tmux pane.

## Chunk 7 — Terminal activation decision

**Goal:** Decide whether Raycast needs to foreground Terminal/iTerm/WezTerm after
Foreman focuses tmux.

**Acceptance Criteria**

- [ ] Manual test from another foreground app documents behavior.
- [ ] If needed, add Raycast preference for terminal app and call Raycast system
      `open`/AppleScript after successful focus.
- [ ] Keep this outside the Foreman Rust binary unless it proves generally useful.

## Validation Ladder

- Fast Rust gate: `mise run check`.
- Focused real tmux smoke for `agents`, `focus`, and `send`.
- Raycast dev smoke: `npm install && npm run dev`, open command, focus pane,
  refresh list, and send sample text.
- Full `mise run verify` only before merging runtime-sensitive changes.

## Open Questions

- Should `agents --json` include only pane rows for MVP, or include session/window
  aggregate rows too?
- Should structured errors use an envelope for every command, or only error
  responses?
- Which terminal app should be the first activation target on this machine
  (Terminal, iTerm2, WezTerm, or current frontmost terminal)?
- Should Raycast live inside this repo long-term, or move to a separate extension
  repo after the spike?
