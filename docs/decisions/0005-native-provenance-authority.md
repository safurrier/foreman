---
id: foreman-adr-0005
title: Native signal authority
description: >
  Defines which hook and activity files may set native agent status.
status: accepted
date: 2026-05-02
index:
  - id: context
    keywords: [native, hooks, fallback, scrollback]
  - id: decision
    keywords: [signals, process, harness]
  - id: consequences
    keywords: [fallback, stale, errors]
  - id: sources
    keywords: [pull-request, tests, commit]
---

# Native signal authority

## Context

Terminal text, pane titles, and tmux pane IDs can outlive an agent. Old text once caused stale labels. A stale file from one harness could also affect another harness in the same pane.

## Decision

Only hooks, events, and activity files may set native status.

Foreman checks the live process and harness before it uses a native signal. Process and terminal clues remain lower-trust fallback data. Users may still select fallback mode in config.

## Consequences

A native signal can correct a fallback label. Terminal text can't create native status on its own. Missing or stale native data falls back with a clear detail.

Each harness uses shared atomic writers and typed readers. Tests cover stale files, process changes, overlapping runs, and cross-harness leaks.

## Sources

- Reasons and test plan — [pull request 10](https://github.com/safurrier/foreman/pull/10), merge `f7bd2ea3eb44b97eb7c65a74348bcc529a602598`.
- Current code at cutoff `5498eba741ce17231be503c31bbf19bffbb5d9f9` — `src/integrations/native.rs`, `src/integrations/mod.rs`, and `src/integrations/AGENTS.md`.
