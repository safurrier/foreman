---
id: macos-popup-raycast-spike-todo
title: Task List
description: >
  Speclab/research pass checklist for Foreman macOS/Raycast popup exploration.
---

# TODO — macos-popup-raycast-spike

## Speclab Pass

- [x] Create branch and plan directory.
- [x] Review Foreman product contract and architecture boundaries.
- [x] Inspect current CLI, tmux adapter, state, and JSON support.
- [x] Run external research on Raycast extension development and command execution.
- [x] Run external research on CLI JSON output practices.
- [x] Write research-backed specification for Foreman JSON/control API.
- [x] Write implementation plan for Raycast extension spike.
- [x] Capture risks and open questions.

## Future Implementation — Foreman Control API

- [ ] Add clap subcommands without regressing existing top-level flags.
- [ ] Add stable public DTOs for `foreman agents --json`.
- [ ] Implement `foreman agents --json`.
- [ ] Implement `foreman focus --pane`.
- [ ] Implement `foreman send --pane --text/--stdin`.
- [ ] Add fake-backed and real-tmux tests.
- [ ] Document the new control API.

## Future Implementation — Raycast Spike

- [ ] Create `integrations/raycast/foreman` extension skeleton.
- [ ] Add `foremanPath` preference.
- [ ] Render searchable List from `foreman agents --json`.
- [ ] Add detail panel for preview/metadata.
- [ ] Add focus action.
- [ ] Add send action.
- [ ] Add refresh action.
- [ ] Manually validate from another foreground macOS app.
- [ ] Decide whether terminal activation belongs in Raycast or Foreman.
