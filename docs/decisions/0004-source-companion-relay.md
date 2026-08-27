---
id: foreman-adr-0004
title: Source companion and snapshot architecture
description: >
  Records the companion protocol, snapshots, reverse-tunnel transport, and trusted remote actions.
status: accepted
date: 2026-06-09
updated: 2026-07-14
index:
  - id: context
    keywords: [sources, companion, snapshots]
  - id: decision
    keywords: [protocol, tunnel, trust]
  - id: consequences
    keywords: [freshness, fallback, runtime]
  - id: alternatives
    keywords: [ssh, relay, daemon]
  - id: invariants
    keywords: [key, trust, atomic]
  - id: checks
    keywords: [probe, smoke, latency]
  - id: sources
    keywords: [pull-request, commit]
related:
  code:
    - src/source_companion.rs
    - src/source_companion_connect.rs
    - src/source_snapshots.rs
    - src/sources.rs
---

# Source companion and snapshot architecture

## Context

One-shot SSH works for occasional remote queries, but repeated popup and native-app refreshes can make interaction slow. It also gives a remote host no direct view of workstation sources.

Foreman needs a faster read path, clear stale-state rules, and a trusted action path. The design must preserve `source_id` and keep failures local to one source.

## Decision

Add a lightweight companion near a source.

- The companion serves schema-versioned JSON-line requests.
- A native probe sends a real protocol request.
- Snapshots provide a prewarmed read path and a fallback when live refresh fails.
- A supervisor can run a local companion and an SSH reverse tunnel.
- Records describe endpoint key and freshness.
- Send capability requires direct enablement and a token.
- Focus keeps tmux and display focus results separate.

Use snapshots for pane list, not as a second source of product truth. Live sources and snapshots share the same source-scoped key model.

### Snapshot ownership

`SourceSnapshotStore` owns snapshot and record files. It uses atomic writes and strict reads. Every record carries a schema version, source ID, and timestamp.

Freshness states have simple behavior:

| State | Behavior |
|---|---|
| fresh | Render normally and refresh in the background. |
| warm | Render with a stale marker and start refresh. |
| stale | Keep rows below healthy data and show a source detail. |
| expired | Hide rows by default and show the detail. |

Corrupt, partial, mismatched, or unsupported files never hide healthy local rows.

### Companion trust

Read-only pane list can run without send authority. Send requires `--allow-send` and a token. Reverse tunnels or future relay transports must preserve the same boundary.

Bind a companion to loopback unless another trusted transport protects it. Empty TCP probes are invalid because they can consume the server's request slot without proving protocol health.

## Consequences

### Benefits

- Popup and native-app startup can use fresh local data.
- Remote hosts can reach workstation sources through a supervised tunnel.
- One source can fail without blocking others.
- Direct freshness and coverage make fallback visible.
- The protocol creates a transport seam without moving source logic into the UI.

### Costs and limits

- Companions and tunnels add runtime state and details.
- Snapshot freshness needs careful policy and reconciliation.
- Trusted send adds token handling and trust checks.
- Cached panes can disappear before an action reaches the live source.
- A shared relay remains future work.

## Alternatives

### Secure shell fallback

This has the fewest moving parts. It remains a fallback, but it can't provide cheap repeated refresh or reverse visibility.

### Local prewarmer without a companion

This improves startup reads but doesn't support live remote actions or reverse queries.

### Shared relay service

A relay could simplify connectivity across many hosts. It also adds deployment, authentication, retention, and service ownership. Foreman deferred it.

### Generic daemon first

A broad daemon would commit to more runtime and API surface than the source problem requires. The companion stays narrow.

## Invariants

- Every row and action retains its source ID.
- Snapshot writes are atomic and source-validated.
- Expired or invalid snapshots can't replace healthy data.
- Send stays disabled without direct trust.
- A protocol probe must parse a valid Foreman response.
- Tmux focus success doesn't depend on display focus.
- Display key remains machine-local.
- Slow refresh never blocks keyboard handling.

## Checks

- Unit tests cover snapshot round trips, corruption, schema mismatch, freshness, pruning, record replacement, and source mismatch.
- Companion tests cover JSON-line requests, tokens, send policy, focus, and protocol errors.
- Reverse-tunnel smoke tests use an isolated tmux server and native companion probes.
- Popup latency tests cover local, all-source idle, and refresh-overlap input bursts.
- macOS fixtures decode source, freshness, details, and action results.

## Sources

- Companion design and alternatives — [pull request 26](https://github.com/safurrier/foreman/pull/26), merge `b14a0abda2abe1cc017a46e6f1d7e6137af1003b`.
- Companion, snapshots, and tunnel code — [pull request 27](https://github.com/safurrier/foreman/pull/27), merge `813f445b6f57b1ab8ec9ab3b64fff2a7663fe004`.
- Exact display key — [pull request 35](https://github.com/safurrier/foreman/pull/35), merge `2ff9cc774fcaf390c22640c931be35dcdea7b5a8`.
- Current code at cutoff `5498eba741ce17231be503c31bbf19bffbb5d9f9` — `src/source_companion.rs`, `src/source_companion_connect.rs`, `src/source_snapshots.rs`, `src/source_display/`, and `src/sources.rs`.
