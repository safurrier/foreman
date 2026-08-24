---
id: foreman-adr-0003
title: ADR 0003—Remote jump and terminal activation
description: >
  Separates remote tmux focus from machine-local terminal display activation.
status: accepted
date: 2026-06-09
updated: 2026-07-14
index:
  - id: context
    keywords: [remote, focus, terminal, display]
  - id: options
    keywords: [activation-command, ghostty, companion]
  - id: decision
    keywords: [tmux-focus, display-activation, fallback]
  - id: validation-requirements
    keywords: [tcc, ownership, diagnostics]
related:
  code:
    - src/runtime.rs
    - src/source_display/
    - src/sources.rs
  docs:
    - docs/operator-guide.md
    - docs/decisions/0002-source-aggregation-and-remote-ssh.md
  issues:
    - https://github.com/safurrier/foreman/issues/34
---

# 0003: Remote Jump and Terminal Activation


## Context

Foreman can now focus remote tmux panes through source-routed actions, e.g.
`foreman --source remote-dev focus --pane %3`. That is only half of the
operator experience. A common setup has Ghostty tabs for both local and
a remote development host, and the expected Foreman behavior is still command-palette-like: select a
row, press Enter/focus, and land in the right pane. A flow that focuses remote
tmux and then asks the operator to manually switch to the remote terminal tab is
not good enough as the target UX.

The implementation also revealed a related problem: noninteractive SSH does not
inherit the `$TMUX` environment from an already-attached Ghostty remote-host tab. A
remote source may query a different tmux server/socket than the tab the operator
is looking at unless the source is configured or registered precisely.

## Options

### A. Remote tmux focus only

Foreman focuses the selected pane on the remote source and reports success. The
operator manually switches to the terminal tab that displays that source.

Pros:

- Already mostly implemented.
- Terminal-independent.
- Safe fallback when terminal activation is unavailable.

Cons:

- Does not meet the desired UX. It requires an extra manual tab switch.

Decision: keep as fallback only.

### B. Configurable activation command

Each source can declare a local activation command. Focus becomes a two-step
operation:

1. Source-routed tmux focus, e.g. `ssh remote-dev tmux switch-client -t %3`.
2. Run a local command to activate the terminal/tab that displays that source.

Example shape:

```toml
[sources.remote-dev.jump]
activate_command = "osascript ~/.config/foreman/focus-coder-ghostty-tab.scpt"
```

Pros:

- First shippable path for a remote-host setup.
- Terminal-agnostic escape hatch.
- Does not require Foreman to own a persistent source daemon.

Cons:

- User-specific and potentially fragile.
- Requires careful placeholder/shell handling and diagnostics.

Decision: recommended first implementation path.

### C. Ghostty AppleScript integration

Ghostty 1.3+ exposes a macOS AppleScript object model with windows, tabs, and
terminals. Foreman can potentially activate Ghostty, select the configured remote-host
tab, and focus its terminal after focusing the remote tmux pane.

Proof/research notes:

- Ghostty docs describe `application -> windows -> tabs -> terminals`.
- `window` has `selected tab`. `tab` has `selected` and `focused terminal`.
- AppleScript commands include selecting a tab and focusing a terminal.
- First use may trigger macOS Automation/TCC permission prompts.
- The API is currently described as preview/young, so Foreman should gate this
  behind explicit config.

Potential shape:

```toml
[sources.remote-dev.jump]
strategy = "ghostty-applescript"
tab_title_contains = "Remote dev"
```

Pros:

- Best native macOS UX if tab identification is reliable.
- Can preserve the single-key Enter/focus interaction.

Cons:

- Ghostty/macOS-specific.
- Stable tab identification needs more proof.
- Splits/windows/titles can make matching ambiguous.

Decision: research/prototype after configurable activation command, then promote
if reliable.

### D. Source companion / registration

A lightweight command or companion process runs inside each source tab and
registers:

- source id
- remote tmux server/socket
- terminal app/window/tab identity
- last heartbeat

Foreman can then route focus precisely and activate the registered display.

Pros:

- Best long-term model for correct jump-to and source endpoint discovery.
- Also helps with cached/streamed remote state.

Cons:

- Larger design, closer to the daemon/bridge direction that is intentionally out
  of scope for the current source aggregation branch.

Decision: keep as long-term direction, not the next implementation.

## Decision

Ship jump-to as two structurally separate outcomes: tmux focus first, then best-effort local display activation. The compatibility activation command remains supported.

ADR 0004 Slice 6 completes the native path with a machine-local source display registry. Foreman captures Ghostty's official stable terminal UUID and activates the exact terminal through its AppleScript `focus` command. Window/tab IDs and title are retained only for diagnostics. Title substring, tty, and pid never select the target. A current registration is attempted before the activation command fallback.

Display identity stays on the machine that owns the display. It is not copied into SSH, snapshot, or companion registration payloads. Companion-host focus reads the companion process's own local registry.

## Validation requirements

Before implementing terminal activation, prove:

- Remote source focus still works without display registration or activation configured.
- Display or activation-command failure does not make successful tmux focus look failed.
- Placeholder expansion is shell-safe.
- Ghostty AppleScript capture and focus use the stable terminal UUID, or report an actionable `source.display.*` diagnostic for provider, platform, TCC, or closed-terminal failures.
- Replacement creates a new opaque ownership handle. Unregister compares the current handle before deletion.
- `sources list`, `sources doctor`, and `sources display doctor` expose local registration health alongside tmux endpoint diagnostics.
