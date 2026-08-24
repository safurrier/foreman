---
id: foreman-adr-0005
title: ADR 0005—Native provenance authority
description: >
  Keeps native agent status grounded in provider hooks or structured files while retaining compatibility fallback.
status: accepted
date: 2026-05-02
index:
  - id: context
    keywords: [native, hooks, compatibility, scrollback]
  - id: decision
    keywords: [provenance, runtime-identity, harness]
  - id: consequences
    keywords: [fallback, stale-signals, diagnostics]
  - id: sources
    keywords: [pull-request, tests, commit]
---

# ADR 0005: Native provenance authority

## Context

Terminal scrollback, pane titles, and reused tmux pane IDs can outlive the agent process that produced them. Treating that compatibility evidence as native state caused stale agent classifications and allowed one harness's old signal file to affect another harness.

## Decision

Treat provider hooks, events, and structured activity files as the only native provenance. Before applying a native signal, verify current or process-derived runtime identity and the matching harness. Keep terminal and process heuristics as lower-confidence compatibility evidence, and honor explicit compatibility-mode configuration.

## Consequences

Native state can correct a compatibility classification, but terminal text never creates native provenance. Missing or stale native data falls back to compatibility behavior with diagnostics instead of dropping the pane. Each provider must use the shared overlay and atomic signal writers, and tests must cover stale-signal rejection and cross-harness isolation.

## Sources

- Rationale and test plan: [PR #10](https://github.com/safurrier/foreman/pull/10), merge `f7bd2ea3eb44b97eb7c65a74348bcc529a602598`.
- Current implementation: `src/integrations/native.rs`, `src/integrations/mod.rs`, and `src/integrations/AGENTS.md` at cutoff `5498eba741ce17231be503c31bbf19bffbb5d9f9`.
