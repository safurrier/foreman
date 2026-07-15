# Integration and E2E Tests

**When a user corrects you or provides you with tribal knowledge/gotchas —
something you could not have known from reading the code or your prompt — you
MUST document it in an AGENTS.md file before continuing.** Write the correction
in the AGENTS.md closest to where the issue occurred.

This directory owns Rust integration targets, real-provider opt-in tests, Python entrypoint contract tests, and shared external-process fixtures. Unit tests stay beside their Rust modules; canonical lane selection remains in `docs/workflows.md`.

## Commands

From the repository root, run `cargo test --test tmux_inventory` to execute one integration target while iterating.

## Gotchas

- **DO** create live tmux worlds with `support::tmux::TmuxFixture`. **NOT** create, inspect, or kill sessions on the user's default tmux server. **BECAUSE** tests must be isolated and must never disrupt an operator's active sessions.

- **DO** use portable `sh` in tmux fixture commands. **NOT** assume `zsh` exists. **BECAUSE** Linux CI runners do not guarantee zsh and panes can exit before assertions begin.

- **DO** wait on pane output, files, logs, or active-pane state with bounded polling helpers. **NOT** use fixed sleeps or assume tmux window indexes. **BECAUSE** server startup and pane scheduling are asynchronous, and indexes vary with fixture topology.

- **DO** keep real Claude, Codex, and Pi tests ignored and run them through the opt-in native lane described in `docs/workflows.md`. **NOT** make provider auth or network access part of the ordinary test suite. **BECAUSE** deterministic hook and tmux tests own the default gate, while real-provider proof requires explicit environment readiness.

- **DO** assert runtime behavior through compiled binaries, structured JSON, and structured logs at external seams. **NOT** reconstruct readiness or provenance from ad hoc terminal text when a machine-facing contract exists. **BECAUSE** terminal output is a compatibility surface and is sensitive to layout and timing.

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-07-15 -->
