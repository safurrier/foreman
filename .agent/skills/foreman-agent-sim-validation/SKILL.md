---
name: foreman-agent-sim-validation
description: Run Foreman repo-local agent simulation validation scenarios for PR-sized behavior checks, especially synthetic tmux/native-signal/Pi-subagent rollouts that should exercise public Foreman CLI surfaces without requiring real long-running agents.
---

# Foreman Agent Simulation Validation

Use this skill when validating Foreman behavior that benefits from a realistic
operator simulation but should remain deterministic, local, and CI-friendly.
These scenarios are **agent simulations**, not dogfood studies: they use
scripted fixtures and public Foreman commands. Reserve **dogfood** for real
worker-prompt or subagent replay studies where agent behavior itself is under
observation.

## Principles

- Run isolated fixtures only. Use temp repos, temp status files, and isolated
  tmux sockets; never touch the user's default tmux server.
- Exercise public surfaces. Prefer `foreman agents`, `foreman extensions`, and
  focused integration tests over unit-only seams.
- Keep scenarios replayable. Each scenario should include setup, command,
  expected observations, and optional collection steps.
- Name deterministic tests `agent_sim`, `smoke`, or `rollout`; do not call them
  dogfood.
- Use real worker prompts only for a separate dogfood/replay skill.

## Scenarios

Current scenarios live under `scenarios/`:

- `pi-subagent-activity/` — synthetic Pi pane plus fresh `pi-subagents` status
  files. Proves native Pi subagent activity promotion, extension cards, and
  one-way workspace containment.

## Standard workflow

1. Pick the scenario matching the changed behavior.
2. Read the scenario `README.md` and `expected-observations.md`.
3. Run `setup.sh` when present.
4. Run the scenario command or test exactly as documented.
5. Run `collect.sh` when present to gather compact observations.
6. Record validation with HK, for example:

   ```bash
   hk validate --why "Agent simulation proves <behavior>." -- <scenario-command>
   ```

7. If adding a new scenario, include at least:

   ```text
   scenarios/<name>/
     README.md
     setup.sh
     collect.sh
     expected-observations.md
   ```

   Add `worker-prompt.md` only when the scenario intentionally asks a real
   worker/subagent to perform a replay. Otherwise keep it scripted.

## Quick commands

Run the Pi subagent activity simulation:

```bash
cargo test --test pi_subagent_agent_sim
```

Run the focused unit seam it depends on:

```bash
cargo test -q pi_subagents
```
