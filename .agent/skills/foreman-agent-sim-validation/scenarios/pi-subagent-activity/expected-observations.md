# Expected observations

The scenario should pass with one Rust integration test:

```text
test agent_sim_pi_subagent_activity_promotes_parent_without_leaking_to_nested_pane ... ok
```

The test asserts these operator-facing observations through public Foreman CLI
JSON output:

- Parent Pi pane:
  - `harness = "pi"`
  - `integrationMode = "native"`
  - `status = "working"`
  - `activeRunCount = 2`
  - `pi-subagents` extension card summary is `2 running`
- Nested module Pi pane:
  - `harness = "pi"`
  - `integrationMode = "native"`
  - `status = "working"`
  - `activeRunCount = 1`
  - `pi-subagents` extension card summary is `1 running`

The parent count proves nested activity rolls up to the parent workspace. The
module count proves repo-root activity does not leak down into a nested module
pane.

Failures usually mean one of these contracts changed:

- Pi runtime recognition for synthetic tmux panes.
- `pi-subagents` status file parsing or freshness filtering.
- Workspace containment direction.
- `foreman agents --json` active-run serialization.
- `foreman extensions --json` Pi subagent card scoping.
