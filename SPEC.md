---
id: foreman-spec
title: Foreman specification
description: >
  Current requirements, interfaces, invariants, and acceptance criteria for Foreman.
index:
  - id: summary
    keywords: [overview, product, tmux, agents]
  - id: requirements
    keywords: [required, recommended, extra, tools, alerts]
  - id: interfaces-contracts
    keywords: [cli, config, tmux, control-api, sources]
  - id: invariants
    keywords: [state, choice, precedence, errors]
  - id: acceptance
    keywords: [checks, scenarios, check, verify]
---

# Foreman specification

## Summary

Foreman is a keyboard-first user console for coding agents in tmux. It shows local and remote work in one source-aware pane list. Users can inspect status, focus panes, send text, manage panes, review pull requests, and receive alerts.

Foreman supports two status paths:

- Native tools read typed hooks, events, or activity files.
- Fallback tools infer state from processes and captured terminal text.

Native signal has more authority. Fallback mode keeps Foreman useful when a harness has no stable native contract.

## Goals / Non-Goals

Goals:

- Make active, idle, blocked, and failed agents easy to distinguish.
- Keep navigation and direct actions fast enough for a command-palette workflow.
- Support local tmux, remote SSH sources, snapshots, and trusted companions.
- Expose one Rust control plane to the terminal UI and `Foreman.app`.
- Fail softly when an extra tool, source, or service is unavailable.

Non-goals:

- Replace tmux, an agent harness, or a full terminal emulator.
- Parse every harness prompt or approval flow.
- Let the Swift app own tmux scan or status logic.
- Treat remote destructive actions as safe without an direct contract.
- Hide source, transport, or source errors from the user.

## Requirements

Foreman enforces every item below unless a rule says otherwise.

### R1. Product and command surface

- Foreman MUST start the live terminal UI when the user runs `foreman` without a utility command.
- `--popup` enables popup rules without creating a second product mode.
- Utility commands expose setup, doctor, config, agents, focus, send, links, sources, companions, extensions, and display registration.
- `--version` prints the version and exits.

### R2. Startup and config

- Foreman starts with a loading shell before the first live pane list arrives.
- It loads config from the resolved path and applies direct command-line overrides.
- It preserves representative version 1 config while applying defaults for newer fields.
- `--init-config` never overwrites an existing file.
- `--doctor` reports config, repo, runtime, and tool problems with useful severity.

### R3. Logging and observability

- Each run writes a log and updates a current-log pointer.
- Logs record pane counts, native signals, fallback use, desktop alerts, pull-request lookup, timing, and errors.
- Retention stays bounded.
- Debug logging remains opt-in.

### R4. Pane list and source key

- Foreman discovers tmux sessions, windows, and panes without assuming one session.
- It hides non-agent panes by default and can reveal them explicitly.
- Every merged pane key combines `source_id` with the source-local tmux pane ID.
- Local, SSH, snapshot, and companion errors stay scoped to their source.
- Healthy sources remain usable when another source fails.

### R5. Tool authority

- Claude, Codex, and Pi each support native and fallback status paths.
- Native signal only applies to the matching harness and current runtime key.
- Stale files, pane titles, and old scrollback can't create native status.
- Missing native signal falls back to fallback mode when config allows it.
- Working-state stabilization can debounce a brief signal loss, but attention and errors surface immediately.

### R6. Status and visibility

- The status model includes working, idle, attention, error, unknown, and non-agent states.
- The UI shows the signal source for the selected pane.
- Users can filter by harness and toggle non-agent sessions or panes.
- Search and filters preserve logical choice when possible.

### R7. Main interface

- The terminal UI includes a header, source-aware sidebar, preview, footer, and contextual overlays.
- The preview shows the resolved useful pane, status source, repo context, pull-request state, extension cards, and user details.
- Help documents every active shortcut and can scroll without moving the sidebar.
- Narrow terminals degrade without panic or hidden critical state.

### R8. Keyboard and pane actions

- Keyboard commands map to typed commands and actions before they reach the reducer.
- Users can navigate, focus, search, use flash labels, compose input, rename windows, spawn windows, and kill panes with confirmation.
- Text input supports multiple lines and Unicode editing.
- Failed send, rename, or spawn actions restore the draft.
- Remote selections reject destructive operations that lack a remote contract.

### R9. Focus and display focus

- Session and window rows resolve to a visible useful pane.
- Tmux focus and local display focus report separate outcomes.
- Tmux focus still succeeds when the extra display step fails.
- Foreman tries a current exact display registration before a focus-command fallback.
- Stable tool IDs select displays. Titles, tty values, and process IDs are detail only.

### R10. Pull requests and extensions

- Pull-request lookup uses the selected pane's linked or detected repo.
- Missing GitHub tooling, authentication, or repo state degrades softly.
- Extension tools run behind bounded, read-only contracts.
- Tool errors remain visible without hiding healthy cards or agent rows.

### R11. Alerts

- Alerts derive from status transitions, not render state.
- Choice, mute state, profile rules, and source-scoped cooldowns can suppress an alert.
- Burst coalescing keeps the most urgent useful signal.
- Backends run in configured order and can fall back.
- Alert errors remain visible and never stop the dashboard.

### R12. Control API and macOS app

- Control commands return schema-versioned JSON.
- The API lists agents, focuses panes, sends text, manages links and sources, and exposes details.
- `Foreman.app` remains a client of that API.
- Swift views don't run tmux commands or infer terminal status.
- The terminal UI and native app share source keys and action semantics.

### R13. Companions, snapshots, and trust

- Companion requests use the documented JSON-line protocol.
- Companion probes send a real protocol request instead of opening an empty socket.
- Send capability requires direct trust and a token.
- Snapshots use atomic writes, source checks, schema checks, and freshness metadata.
- Stale snapshots can support display with a detail. Expired or corrupt snapshots can't hide healthy local rows.
- Reverse-tunnel supervision reports transport errors and keeps local operation ready.

### R14. State ownership

- `AppState` owns interaction state, drafts, modes, choice, cached cards, alert policy state, and user alerts.
- The reducer stays pure and emits clear effects.
- Adapters and services own I/O.
- Rendering reads state and doesn't mutate product rules.
- Persisted UI choices restore safe choices, while direct config wins.

### Recommended rules

Foreman should use native tools when stable contracts exist. It should preserve fast keyboard feedback, useful text-only output, and clear degraded-state details. It should add fields to machine-readable schemas when fallback allows that approach.

### Extra rules

Foreman may show summaries, child-task activity, system pressure, themes, and other derived context when those features don't weaken core actions or signal boundaries.

## Interfaces & Contracts

### Glossary

- **Agent pane.** A pane known as a supported coding agent.
- **Fallback mode.** Status inferred from process and terminal signal.
- **Control API.** JSON commands used by native clients and automation.
- **Native mode.** Status grounded in typed hook signals.
- **User.** The person controlling Foreman.
- **Popup mode.** A tmux-popup surface that closes after valid focus actions.
- **Source.** A local or remote tmux-backed pane list tool.
- **Source-scoped pane.** A pane identified by both source and tmux pane ID.

### Command-line contract

The root command accepts live options and utility subcommands. Control subcommands reject conflicting live modes. Use each command's `--help` output as the detailed option contract.

Important machine-readable commands include:

```text
foreman agents --json
foreman focus --pane <pane-id> --json
foreman send --pane <pane-id> --stdin --json
foreman links list --json
foreman sources list --json
foreman companion probe --endpoint <host:port> --json
```

Focus results keep source-host focus in `displayActivation` and requesting-host focus in `callerDisplayActivation`.

### Config contract

Config covers UI choices, alerts, tools, sources, source jumps, companions, and runtime paths. Unknown or invalid values produce useful errors. New extra fields keep compatible defaults.

### Tmux and source contracts

Tmux scan returns typed pane list or a typed unavailable error. Source aggregation preserves source key, details, and partial success. Actions route through the selected source rather than a bare pane ID.

### Native tool contract

Tool hooks and activity files use shared atomic writers and typed readers. Native signal must match the tool and active runtime key. Fallback signal remains ready as a lower-confidence fallback.

### macOS contract

AppKit owns lifecycle, windows, menus, hotkeys, and app focus. SwiftUI owns visible native UI state. Rust owns tmux, sources, status, and actions through the control API.

### Checks contract

`mise run check` is the fast gate. `mise run verify` is the broad pre-merge gate. Native hook changes also require native preflight and real-harness proof. Overlay changes use the macOS overlay checks lane.

## Invariants

- A choice always resolves through visible source-scoped state.
- Native signal never crosses harness boundaries.
- Fallback signals never become native status.
- The reducer performs no I/O.
- Source errors don't erase healthy source results.
- Tmux focus success doesn't depend on display focus success.
- Display key stays machine-local and outside source transport payloads.
- Remote send requires direct authorization.
- Generated text isn't sent without a direct user action.
- Pull-request and extension errors don't block core tmux control.
- Checks never create or kill sessions on the user's default tmux server.

## Acceptance

### Checks entrypoints

The repo must keep these entrypoints working:

```text
mise run check
mise run verify-agent-entrypoints
mise run verify-ux
mise run validate-macos-overlay-change
mise run native-preflight
mise run verify-native
```

### Core scenarios

- Startup renders a loading shell and then a live pane list.
- Multi-session scan preserves hierarchy and useful pane resolution.
- Native status overrides matching fallback state and rejects stale or cross-harness signals.
- Filters, search, sorting, and refreshes preserve logical choice.
- Focus routes through the selected source and reports display warnings separately.
- Compose, rename, spawn, and kill flows preserve drafts or confirmation on failure.
- Pull-request, extension, alert, and source errors degrade without stopping the dashboard.
- Companion probes, tokens, snapshots, and reverse tunnels preserve their trust boundaries.
- The native app decodes the current control schema and delegates every tmux action to Rust.
- Version 1 config fixtures load with current defaults.

### Definition of done

A change is complete when focused tests cover its invariant, the required repo gate passes, documentation and contracts match current rules, and no unresolved review finding proves the accepted rules false.
