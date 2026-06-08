---
title: ADR 0003 — Remote jump and terminal activation
summary: Decide how Foreman should focus both a remote tmux pane and the local terminal tab that displays that source.
status: proposed
updated: 2026-06-08
related:
  code:
    - src/runtime.rs
    - src/sources.rs
  docs:
    - docs/operator-guide.md
    - docs/decisions/0002-source-aggregation-and-remote-ssh.md
---

# 0003: Remote Jump and Terminal Activation

Status: proposed

## Context

Foreman can now focus remote tmux panes through source-routed actions, e.g.
`foreman --source coder-dev-gpu-1 focus --pane %3`. That is only half of the
operator experience. Alex's normal setup has Ghostty tabs for both local and
Coder, and the expected Foreman behavior is still command-palette-like: select a
row, press Enter/focus, and land in the right pane. A flow that focuses remote
tmux and then asks the operator to manually switch to the Coder terminal tab is
not good enough as the target UX.

The implementation also revealed a related problem: noninteractive SSH does not
inherit the `$TMUX` environment from an already-attached Ghostty Coder tab. A
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

- Does not meet the desired UX; it requires an extra manual tab switch.

Decision: keep as fallback only.

### B. Configurable activation command

Each source can declare a local activation command. Focus becomes a two-step
operation:

1. Source-routed tmux focus, e.g. `ssh coder tmux switch-client -t %3`.
2. Run a local command to activate the terminal/tab that displays that source.

Example shape:

```toml
[sources.coder-dev-gpu-1.jump]
activate_command = "osascript ~/.config/foreman/focus-coder-ghostty-tab.scpt"
```

Pros:

- First shippable path for Alex's setup.
- Terminal-agnostic escape hatch.
- Does not require Foreman to own a persistent source daemon.

Cons:

- User-specific and potentially fragile.
- Requires careful placeholder/shell handling and diagnostics.

Decision: recommended first implementation path.

### C. Ghostty AppleScript integration

Ghostty 1.3+ exposes a macOS AppleScript object model with windows, tabs, and
terminals. Foreman can potentially activate Ghostty, select the configured Coder
tab, and focus its terminal after focusing the remote tmux pane.

Proof/research notes:

- Ghostty docs describe `application -> windows -> tabs -> terminals`.
- `window` has `selected tab`; `tab` has `selected` and `focused terminal`.
- AppleScript commands include selecting a tab and focusing a terminal.
- First use may trigger macOS Automation/TCC permission prompts.
- The API is currently described as preview/young, so Foreman should gate this
  behind explicit config.

Potential shape:

```toml
[sources.coder-dev-gpu-1.jump]
strategy = "ghostty-applescript"
tab_title_contains = "Coder"
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

Ship the next jump-to slice in this order:

1. Keep remote tmux focus as a fallback.
2. Add a configurable per-source activation command with clear diagnostics.
3. Prototype Ghostty AppleScript as an explicit strategy for Alex's setup.
4. Revisit source companion registration only if configurable/Ghostty activation
   proves too brittle or if Foreman needs persistent source state for other
   reasons.

## Validation requirements

Before implementing terminal activation, prove:

- Remote source focus still works without activation configured.
- Activation command failure does not make remote tmux focus look failed.
- Placeholder expansion is shell-safe.
- Ghostty AppleScript proof commands work on the target macOS/Ghostty version, or
  the feature reports an actionable unavailable diagnostic.
- `sources doctor` can show enough tmux endpoint detail to explain why a Coder
  tab and noninteractive SSH source might see different sessions.
