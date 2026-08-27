---
id: foreman-adr-0003
title: Remote focus and local terminal switching
description: >
  Separates source-routed tmux focus from machine-local terminal display switching.
status: accepted
date: 2026-06-09
updated: 2026-07-14
index:
  - id: context
    keywords: [remote, focus, terminal, display]
  - id: decision
    keywords: [tmux, switching, fallback]
  - id: consequences
    keywords: [key, details, platform]
  - id: alternatives
    keywords: [manual, command, ghostty, companion]
  - id: checks
    keywords: [owner, tcc, failure]
  - id: sources
    keywords: [pull-request, commit]
related:
  code:
    - src/runtime.rs
    - src/source_display/
    - src/sources.rs
  docs:
    - docs/user-guide.md
    - docs/decisions/0002-source-aggregation-and-remote-ssh.md
---

# Remote focus and local terminal switching

## Context

Focusing a remote tmux pane doesn't bring its local terminal tab to the front. Users then have to switch tabs by hand, which breaks Foreman's command-palette flow.

Terminal switching also belongs to the machine that owns the display. A remote source can't safely choose a local window from a title, tty, or process ID. Those values can be stale or ambiguous.

## Decision

Report focus as two separate outcomes:

1. Route tmux focus to the selected source and pane.
2. Try machine-local display switching as an extra follow-up.

Tmux focus still succeeds when display switching fails. Foreman reports the display warning separately.

Each machine may register one exact display key per source. Foreman tries that record first. It can then run a configured switching-command fallback. Stable app IDs select a display. Titles, tty values, and process IDs remain detail only.

Display records use opaque owner handles. Replacement creates a new handle, and stale cleanup can't remove a newer record. Display key never enters SSH, snapshot, or companion transport payloads.

Example fallback:

```toml
[sources.remote-dev.jump]
activate_command = "osascript ~/.config/foreman/focus-remote-terminal.scpt"
```

## Consequences

### Benefits

- One action can focus tmux and raise the correct local terminal.
- A closed terminal doesn't turn successful tmux focus into failure.
- Exact app key avoids fragile title matching.
- Source transports stay independent from local display state.

### Costs and limits

- Native switching depends on app and platform support.
- macOS may require Automation permission.
- Fallback commands need safe placeholder expansion and timeouts.
- Users must refresh stale records after a display closes or changes key.

## Alternatives

### Remote tmux focus only

This remains the safe fallback, but it requires a manual tab switch.

### Switching command only

This is portable and easy to configure, but command scripts can be fragile. Foreman keeps it as a fallback path.

### Title-based native matching

Titles change and can collide. Foreman rejected them as selectors but keeps them for details.

### Persistent companion owner

A companion can activate the display on its own host. That path uses the same machine-local registry and doesn't move display key across the network.

## Checks

- Focus works without display config.
- Display failure leaves tmux success intact.
- Record replacement and cleanup enforce owner handles.
- Ghostty capture and focus use the stable terminal `UUID`.
- Unsupported platforms and permission failures return typed details.
- Switching commands have bounded execution time.

## Sources

- Source-aware focus baseline — [pull request 25](https://github.com/safurrier/foreman/pull/25), merge `07d3e9793d45798cc0c38fd3e3a84b75edd32657`.
- Exact display key — [pull request 35](https://github.com/safurrier/foreman/pull/35), merge `2ff9cc774fcaf390c22640c931be35dcdea7b5a8`.
- Current code at cutoff `5498eba741ce17231be503c31bbf19bffbb5d9f9` — `src/source_display/`, `src/runtime.rs`, and `src/sources.rs`.
