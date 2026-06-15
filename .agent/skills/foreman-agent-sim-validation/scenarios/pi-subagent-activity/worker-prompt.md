# Optional worker replay prompt

Use this only when intentionally running a real worker/subagent replay around the
scripted scenario. For ordinary validation, run `collect.sh` directly instead.

```text
You are validating Foreman's Pi subagent activity simulation in a temporary or
PR worktree. Do not modify source files unless the scenario fails and the parent
explicitly asks for a fix.

Steps:
1. Read .agent/skills/foreman-agent-sim-validation/scenarios/pi-subagent-activity/README.md.
2. Run setup.sh for the scenario.
3. Run collect.sh for the scenario.
4. Compare the output to expected-observations.md.
5. Write a concise report with:
   - commands run
   - pass/fail result
   - any mismatch from expected observations
   - whether this looked like a deterministic agent_sim failure or a broader
     Foreman behavior regression
```
