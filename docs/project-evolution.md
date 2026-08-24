---
id: project-evolution
title: Project Evolution
description: >
  Evidence-backed phases that explain how Foreman grew from a local tmux dashboard into a source-aware terminal and macOS control plane.
index:
  - id: phases
    keywords: [history, releases, sources, companion, macos]
  - id: retained-boundaries
    keywords: [invariants, provenance, ownership]
---

# Project evolution

## Phases

- **2026-04-08—Contract-first scaffold:** The initial commit established the Rust/Ratatui stack, product specification, architecture record, and stack decision.
- **2026-04-23—Local tmux operator surface:** Pull request 1 completed the reducer-driven dashboard, tmux adapter, notifications, pull-request lookup, tests, and first release evidence.
- **2026-05-02—Native provenance:** Pull request 10 made provider hooks and structured files authoritative while retaining lower-confidence compatibility fallback.
- **2026-05-07—Native macOS surface:** Pull request 14 added Foreman.app as a typed client of the Rust control API rather than a second owner of tmux truth.
- **2026-06-09—Source-aware inventory:** Pull request 25 introduced source-scoped identity, local and SSH aggregation, and action routing across terminal and macOS surfaces.
- **2026-06-10—Companion and snapshots:** Pull requests 26 and 27 added the companion protocol, prewarmed snapshots, reverse-tunnel supervision, and failure fallback.
- **2026-07-14—Display activation and unattended entrypoints:** Pull requests 35 and 36 established machine-local display identity plus versioned setup and resume receipts.
- **2026-07-17 onward—Scoped guidance and compatibility:** Pull request 37 distributed subsystem guidance. Pull request 40 pinned representative version 1 configuration compatibility.

## Retained boundaries

Foreman keeps interaction state in the reducer and I/O in effects and adapters. Provider-native provenance remains separate from compatibility heuristics. Pane identity stays scoped by source, and machine-local display identity stays outside source transport payloads. Historical pull-request rationale and current tests support these boundaries. Release and workflow artifacts remain evidence rather than product authorities.
