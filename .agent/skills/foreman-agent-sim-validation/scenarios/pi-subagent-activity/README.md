# Pi subagent activity agent simulation

Purpose: validate Foreman's public CLI behavior when a Pi parent pane has fresh
structured `pi-subagents` activity.

This is a deterministic agent simulation, not dogfood. It uses isolated tmux
fixtures and synthetic `status.json` files to prove the Foreman-side contract
without launching real long-running agents.

## Behavior under test

- A Pi pane at a repo root can inherit active subagent runs from nested module
  directories.
- A nested module Pi pane does not inherit unrelated repo-root subagent runs.
- Foreman reports native Pi status through `foreman agents --json`.
- Foreman exposes a read-only `pi-subagents` extension card through
  `foreman extensions --json`.

## Command

```bash
cargo test --test pi_subagent_agent_sim
```

## Focused unit seam

```bash
cargo test -q pi_subagents
```

## Expected evidence

Record the scenario as HK validation for PR-sized changes that touch Pi native
status, subagent activity parsing, extension-card scoping, or workspace matching:

```bash
hk validate --why "Agent simulation proves Pi subagent native activity and extension-card behavior." -- cargo test --test pi_subagent_agent_sim
```
