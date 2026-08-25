---
id: project-evolution
title: Project evolution
description: >
  Evidence-backed phases in Foreman's product and architecture.
index:
  - id: early-product
    keywords: [specification, tmux, terminal]
  - id: native-signals-and-second-surface
    keywords: [hooks, provenance, macos]
  - id: source-aware-operation
    keywords: [ssh, companion, snapshots]
  - id: current-boundaries
    keywords: [identity, ownership, validation]
---

# Project evolution

## Early product

### Contract-first scaffold, April 8, 2026

The first commit set the Rust and Ratatui stack. It also added the product spec, design record, and stack choice.

### Local tmux console, April 23, 2026

`PR 1` completed the state-driven dashboard. It added tmux actions, alerts, pull-request lookup, tests, and release proof.

## Native signals and second surface

### Native provenance, May 2, 2026

`PR 10` made hooks and activity files the trusted source. Terminal clues stayed as a lower-trust fallback.

### Native macOS app, May 7, 2026

`PR 14` added `Foreman.app`. The app used the Rust control API instead of creating a second tmux layer.

### Extensions and linked repositories, May 15, 2026

`PR 17` added read-only extension cards and direct pane-to-repo links. Card errors stayed apart from core tmux control.

## Source-aware operation

### Merged source inventory, June 9, 2026

`PR 25` added source keys and one merged local or SSH pane list. Both user interfaces gained the same source and action rules.

### Companion and snapshots, June 10, 2026

`PR 26` and `PR 27` added snapshots, a `JSON-line` companion, a watched reverse tunnel, real probes, and clear fallback state.

### Pi subagent activity, June 15, 2026

`PR 29` turned Pi subagent events into native signals. Parent status could now reflect active or blocked child work.

## Current boundaries

### Exact display identity and unattended entrypoints, July 14, 2026

`PR 35` added exact display records on each machine. `PR 36` made setup and resume emit versioned receipts.

### Scoped repository guidance, July 17, 2026

`PR 37` moved module guidance into nested agent files. The root kept repo-wide workflow and safety rules.

### Config fallback, August 10, 2026

`PR 40` pinned a version 1 config sample. Current defaults now load without breaking older settings.

Foreman still keeps several early boundaries. The reducer owns state changes. Adapters own I/O. Native signals stay separate from terminal fallback. Pane keys stay source-scoped. Display keys stay on one machine. Old workflow files remain evidence, not current policy.
