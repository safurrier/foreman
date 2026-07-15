# External Adapters

**When a user corrects you or provides you with tribal knowledge/gotchas —
something you could not have known from reading the code or your prompt — you
MUST document it in an AGENTS.md file before continuing.** Write the correction
in the AGENTS.md closest to where the issue occurred.

This module owns typed process boundaries for external systems; it currently contains Foreman's tmux transport and inventory adapter. Harness interpretation belongs in `src/integrations/`, while runtime scheduling and effect execution stay outside this module.

## Commands

From the repository root, run `cargo test --lib adapters::tmux::tests` for adapter unit tests.

## Gotchas

- **DO** thread `TmuxTarget` through every tmux command. **NOT** fall back to the default tmux server for socket- or server-name-backed operations. **BECAUSE** discovery and actions must address the same isolated or configured server.

- **DO** keep harness recognition and status rules in `src/integrations/`. **NOT** add provider-specific heuristics to tmux parsing or command execution. **BECAUSE** the adapter owns transport and metadata, not status authority.

- **DO** preserve per-pane capture failures and preview provenance while continuing inventory assembly. **NOT** turn one failed `capture-pane` call into a whole-inventory failure or silently label reused text as fresh. **BECAUSE** tmux output is partial and stale by nature, but the dashboard must remain usable and explain preview freshness.

- **DO** use tmux buffers for arbitrary composed input and return typed action results. **NOT** interpolate multiline text into `send-keys` or leak raw subprocess output into reducer-facing code. **BECAUSE** quoting and non-ASCII input must survive the process boundary, and callers need stable success/failure data.

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-07-15 -->
