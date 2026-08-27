---
id: foreman-adr-0001
title: Stack choice
description: >
  Records the Rust stack and the tradeoffs behind it.
status: accepted
date: 2026-04-08
index:
  - id: context
    keywords: [stack, terminal, distribution]
  - id: decision
    keywords: [rust, ratatui, clap, serde, tokio]
  - id: consequences
    keywords: [tooling, build, contributors]
  - id: alternatives
    keywords: [python, go]
---

# Stack choice

## Context

Foreman needs a fast terminal UI and safe state changes. It also needs one small binary and strong tests. The main language sets the build and release tools.

## Decision

Use Rust with these core libraries:

| Concern | Choice |
|---|---|
| Terminal UI | Ratatui and Crossterm |
| Command line | Clap |
| Data and config | Serde, JSON, and `TOML` |
| Errors | anyhow and thiserror |
| Async work | Tokio |
| System metrics | sysinfo |

Use standard Rust tools for formatting, linting, checks, tests, and release builds.

## Consequences

### Benefits

- The type system catches many state and API mistakes before runtime.
- Ratatui fits reducer-driven terminal rendering.
- One release binary keeps setup simple.
- Standard tools keep local and CI checks alike.

### Costs

- Cold builds take longer than an interpreted edit loop.
- Contributors need a Rust toolchain.
- Native macOS UI still needs a separate Swift target.

## Alternatives

Python would make the first draft faster, but it catches fewer state errors before a run. Go would make builds simple, but it fits Ratatui and the reducer model less well.
