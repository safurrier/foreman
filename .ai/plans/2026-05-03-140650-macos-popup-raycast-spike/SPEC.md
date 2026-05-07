---
id: macos-popup-raycast-spike-spec
title: macOS Popup + Raycast Spike Specification
description: >
  Research-backed specification for exposing Foreman through a JSON/control CLI
  and a Raycast extension spike before considering a native macOS popup app.
---

# Specification — macos-popup-raycast-spike

## Problem

Foreman already answers “which agent pane needs me?” inside a terminal/TUI and
tmux popup. The next product question is whether the same operator surface can be
used from anywhere on macOS, with a Raycast-style command palette for quickly
finding an agent pane, previewing recent output, focusing it, and optionally
sending text.

The first two increments should validate the idea without committing Foreman to a
native macOS app framework:

1. Add a stable machine-readable Foreman control surface.
2. Build a Raycast extension spike that consumes that control surface.

## Goals

- Make Foreman state scriptable by external UIs without linking to Ratatui.
- Keep the existing `foreman` TUI behavior unchanged when no subcommand/utility
  flag is provided.
- Let a Raycast command list current agent panes, filter them, show status and
  preview details, and focus/send to a selected tmux pane.
- Learn whether “Foreman from anywhere” is valuable before building a custom
  Swift/Tauri/AppKit popup.

## Non-Goals

- Do not build a custom native macOS app in this slice.
- Do not add GUI dependencies to the main Foreman TUI binary path.
- Do not make Raycast a required runtime dependency for Foreman.
- Do not replace the tmux popup mode; this is an adjacent frontend.
- Do not mutate pull requests or harness state from Raycast.

## Requirements

### P0 — Foreman JSON/control CLI

- `foreman agents --json` returns a documented, stable JSON payload on stdout.
- The payload includes a schema/version field, inventory summary, and a flat list
  of actionable entries suitable for a palette UI.
- Each entry includes at least: pane id, session/window labels, pane title,
  harness kind, status, status source/provenance, current command,
  working directory, preview text, and timestamps/rank if available.
- The command defaults to agent panes only, matching Foreman’s current default
  visibility rules, with flags to include non-agent sessions/panes later if
  needed.
- `foreman focus --pane <PANE_ID>` focuses tmux on an existing pane and exits
  zero on success.
- `foreman send --pane <PANE_ID> --text <TEXT>` sends text using Foreman’s tmux
  input adapter. It should also support stdin, e.g. `--stdin`, before Raycast
  exposes arbitrary user text.
- Machine-readable modes write only JSON/data to stdout; logs, warnings, and
  human error context go to stderr.
- Failures use non-zero exit codes and structured JSON errors when `--json` is
  requested.

### P1 — Raycast extension spike

- Add a private/dev extension under `integrations/raycast/foreman` or equivalent.
- The default command renders a searchable `List` of Foreman agent panes.
- List rows show recognizable title/subtitle/accessories: session/window, status,
  harness, and workspace.
- A detail panel renders recent preview text and key metadata.
- Primary action focuses the selected pane by invoking `foreman focus --pane`.
- Secondary action sends text to the selected pane through `foreman send`.
- Refresh action re-runs `foreman agents --json`.
- Extension preferences allow configuring the `foreman` binary path and optional
  terminal app activation behavior.
- Errors surface through Raycast toasts with enough context to run
  `foreman --doctor`.

### P2 — Follow-on exploration

- Add an optional `foreman open --pane <PANE_ID>` or Raycast-side terminal
  activation helper if focusing tmux is not enough when launched from another
  foreground app.
- Consider a persistent `foreman serve`/local socket only if repeated one-shot
  CLI calls are too slow for Raycast.
- Consider promoting the Raycast extension to a separate repository or package if
  it develops release needs independent from Foreman.

## External Research Findings

- Raycast recommends creating an extension from its Create Extension command and
  running `npm install && npm run dev` for local development and hot reload.
  Source: https://developers.raycast.com/basics/create-your-first-extension
- Raycast `List` supports built-in fuzzy filtering, loading state, dropdowns,
  item actions, and a right-side detail panel via `List.Item.Detail`; this maps
  directly to Foreman’s “agent rows + preview” product shape.
  Source: https://developers.raycast.com/api-reference/user-interface/list
- Raycast `useExec` can execute local commands and parse stdout. Its docs prefer
  the file-plus-arguments form over shell strings and warn that `shell` is slower
  and potentially unsafe. This supports a design where the extension calls
  `foreman` with argument arrays rather than interpolated shell commands.
  Source: https://developers.raycast.com/utilities/react-hooks/useexec
- CLI guidelines recommend machine-readable JSON via `--json`, primary output on
  stdout, logs/errors on stderr, and correct exit codes. This supports treating
  Foreman’s JSON/control commands as a small public API rather than scraping TUI
  output. Source: https://clig.dev/#output

## Codebase Findings

- The current CLI is a single `clap::Parser` struct with utility flags and no
  public subcommand enum yet (`src/cli.rs:35`, `src/cli.rs:41`).
- Foreman already has JSON infrastructure for doctor output
  (`src/cli.rs:144`, `src/cli.rs:489`) and depends on `serde_json`
  (`Cargo.toml:14`).
- Bootstrap already produces an `AppState` and `InventorySummary` that can seed a
  one-shot JSON command (`src/cli.rs:706`, `src/cli.rs:1081`,
  `src/app/state.rs:1134`).
- The tmux adapter already exposes focus and send seams that are fake-testable
  (`src/adapters/tmux.rs:132`, `src/adapters/tmux.rs:306`,
  `src/adapters/tmux.rs:310`).
- The visible target cache already computes actionable pane ids for session,
  window, and pane rows (`src/app/state.rs:1209`, `src/app/state.rs:1777`).
- Architecture requires rendering to stay pure and all tmux/browser/notification
  effects to flow through adapters/services, so the JSON/control surface should
  reuse adapters rather than renderer state (`docs/architecture.md:122`,
  `docs/architecture.md:166`).

## Constraints and Risks

- `tmux switch-client` changes tmux’s selected client/session/window/pane but does
  not necessarily bring the terminal app to the foreground when invoked from
  Raycast. The spike should either document this or add optional Raycast-side
  terminal activation.
- One-shot inventory capture can be slower than an always-running Foreman runtime;
  measure before designing a daemon.
- Adding clap subcommands must not break existing top-level flags like
  `foreman --doctor`, `foreman --setup`, or plain `foreman` startup.
- Raycast extensions are TypeScript/Node projects, so this adds a second toolchain
  under an integration boundary.
- Preview text may include sensitive terminal content; the Raycast detail view
  should not persist or upload it.

## Success Criteria

- `foreman agents --json | jq` produces stable data with no human/log output on
  stdout.
- `foreman focus --pane %N` and `foreman send --pane %N --text ...` work against
  the same fake-backed and real-tmux paths as the TUI actions.
- A Raycast dev command can list active Foreman agent panes and focus one in no
  more than one action after opening Raycast.
- The design leaves room for a future native macOS popup without coupling GUI
  code into Ratatui rendering.
