---
id: foreman-adr-0002
title: Source aggregation and remote SSH targets
description: >
  Defines source-scoped identity, merged local and remote pane list, and one-shot SSH transport.
status: accepted
date: 2026-06-04
updated: 2026-06-09
index:
  - id: context
    keywords: [sources, remote, ssh, tmux]
  - id: decision
    keywords: [identity, aggregation, actions]
  - id: consequences
    keywords: [latency, details, parity]
  - id: alternatives
    keywords: [manual, mac-only, daemon]
  - id: checks
    keywords: [tests, smoke, performance]
  - id: sources
    keywords: [pull-request, commit]
---

# Source aggregation and remote SSH targets

## Context

Foreman first assumed that every pane belonged to the local tmux server. That model breaks when an user works across a laptop, workstation, or remote development host. Tmux pane IDs are only unique inside one server, and each remote source can fail or respond at a different speed.

Both product surfaces need the same model. The terminal popup and `Foreman.app` should show one pane list and route actions to the selected source. Swift shouldn't implement a separate SSH stack.

The team proposed this record on 2026-06-04. Pull request 25 completed the source-aware code on 2026-06-09.

## Decision

Treat every tmux-backed location as a named source.

- Identify a pane with `source_id` plus tmux pane ID.
- Aggregate enabled sources into one pane list.
- Keep a local source for the current machine.
- Use bounded SSH commands without prompts for remote probes and supported actions.
- Query sources in parallel.
- Keep failures and timing details scoped to one source.
- Preserve useful cached rows with an direct stale marker when policy allows it.
- Route focus and send through the selected source.
- Reject remote destructive actions until they have an direct contract.

Rust owns source config, aggregation, identity, details, and actions. The terminal UI and native app consume the same control API.

Example config:

```toml
[sources.local]
kind = "local"
label = "Local"
enabled = true

[sources.remote-dev]
kind = "ssh"
label = "Remote dev"
host = "remote-dev.example"
foreman = "/usr/local/bin/foreman"
tmux_server_name = "user"
enabled = true
```

Example actions:

```bash
foreman agents --json --sources all
foreman focus --source remote-dev --pane %42 --json
foreman send --source remote-dev --pane %42 --stdin --json
```

## Consequences

### Benefits

- Users see local and remote work together.
- Duplicate pane IDs can't collide across sources.
- One failed source doesn't hide healthy rows.
- Both user interfaces share pane list and action semantics.
- The provider boundary leaves room for snapshots and companions.

### Costs and limits

- Remote refresh adds latency and partial-failure states.
- SSH requires a working remote Foreman binary and the correct tmux endpoint.
- Cached data needs freshness labels and careful reconciliation.
- Initial remote actions stay deliberately narrow.

## Alternatives

### Manual source switching

This keeps the code small but hides cross-host work and adds user steps. It remains useful as an direct scope control, not as the main model.

### Native-app-only aggregation

This would split product behavior and duplicate transport logic in Swift. Foreman rejected it.

### Start with a daemon or relay

A service could lower refresh latency, but it adds lifecycle and trust costs before tests prove the source model. The source provider seam supports that future transport without requiring it now.

## Checks

- Unit tests cover source IDs, duplicate pane IDs, parallel aggregation, partial failure, SSH quoting, timeouts, and schema mismatch.
- Control API tests cover merged pane list and source-aware actions.
- Live smoke tests use isolated tmux servers and bounded remote commands.
- Popup performance tests cover local, idle all-source, and refresh-overlap input latency.

## Sources

- Decision and code rationale — [pull request 25](https://github.com/safurrier/foreman/pull/25), merge `07d3e9793d45798cc0c38fd3e3a84b75edd32657`.
- Current code at cutoff `5498eba741ce17231be503c31bbf19bffbb5d9f9`: `src/sources.rs`, `src/runtime.rs`, `src/app/state.rs`, and `src/services/control_api.rs`.
