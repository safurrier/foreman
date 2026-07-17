# Harness Integrations

**When a user corrects you or provides you with tribal knowledge/gotchas —
something you could not have known from reading the code or your prompt — you
MUST document it in an AGENTS.md file before continuing.** Write the correction
in the AGENTS.md closest to where the issue occurred.

This module recognizes supported harnesses, derives compatibility status, bridges provider events, and overlays native signals onto inventory. `SPEC.md` owns supported behavior and status semantics; `docs/workflows.md` owns real-provider validation policy.

## Commands

From the repository root, run `cargo test --lib integrations::` for integration unit tests.

## Gotchas

- **DO** keep native status sourced only from provider hook, event, or structured activity files. **NOT** promote terminal text heuristics to native provenance. **BECAUSE** native status must describe a provider signal, while terminal parsing is explicitly lower-confidence compatibility behavior.

- **DO** verify current or process-derived runtime identity before applying a pane's native signal. **NOT** let a stale pane-id file or old title resurrect a shell/editor pane or override another harness. **BECAUSE** tmux pane ids and titles can outlive the agent process that created the signal.

- **DO** use the shared atomic writers for hook outputs and any fixture updated while refresh runs. **NOT** overwrite a polled JSON file in place. **BECAUSE** runtime refresh can otherwise read a partial payload and report a false fallback or warning.

- **DO** keep native-over-compatibility precedence in the shared overlay layer and honor configured compatibility mode. **NOT** bury precedence in provider phrase matching or the tmux adapter. **BECAUSE** every provider must follow the same provenance rule.

- **DO** test recognition, status explanation, stale-signal rejection, and fallback when changing provider tokens or events. **NOT** add broad terminal phrases without regression cases. **BECAUSE** captured scrollback contains stale errors and prompts that can misclassify unrelated panes.

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-07-15 -->
