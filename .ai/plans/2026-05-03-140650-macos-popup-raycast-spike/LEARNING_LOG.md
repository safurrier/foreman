---
id: macos-popup-raycast-spike-learning-log
title: Learning Log
description: >
  Dev diary for the macOS popup/Raycast speclab pass.
---

# Learning Log

## 2026-05-03

User asked for a speclab-style pass, with external research, on two proposed
increments: (1) a Foreman JSON/control API and (2) a Raycast extension spike for
using Foreman from anywhere on macOS.

Key findings:

- Foreman already has most of the domain seams needed: tmux inventory, visible
  target/actionable pane cache, focus/send adapter methods, and JSON precedent
  through doctor output.
- The cleanest architecture is not a macOS UI inside Ratatui. It is a stable
  one-shot control API plus an external Raycast frontend.
- Raycast maps well to the product: `List` for agent panes, `List.Item.Detail`
  for preview/metadata, `ActionPanel` for focus/send/refresh, and `useExec` for
  calling the local `foreman` binary.
- The main risk is focus semantics. `tmux switch-client/select-pane` changes tmux
  state, but a Raycast action launched from another app may also need to bring
  Terminal/iTerm/WezTerm to the foreground. Keep that as a spike question rather
  than baking macOS app activation into the first Rust control API.
