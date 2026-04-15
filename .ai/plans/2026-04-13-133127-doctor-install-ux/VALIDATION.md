---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

## Investigation evidence - 2026-04-13

This slice is still in planning. No code changes have been made yet.

Commands run during investigation:

```bash
command -v foreman foreman-claude-hook foreman-codex-hook foreman-pi-hook
foreman --config-path
ls ~/.local/state/foreman
tail -n 120 ~/.local/state/foreman/logs/latest.log
tmux list-panes -a -F '#{session_name}\t#{window_name}\t#{pane_id}\t#{pane_current_command}\t#{pane_current_path}\t#{pane_title}'
tmux capture-pane -t %281 -p -S -120
find /Users/alex.furrier/git_repositories -maxdepth 3 \\( -name hooks.json -o -name settings.json -o -name settings.local.json -o -name foreman.ts \\)
sed -n '1,220p' README.md
sed -n '1,260p' src/cli.rs
sed -n '1,260p' src/integrations/native.rs
```

Observed local state:

- Binaries are installed and on `PATH`.
- No user config exists at the resolved config path.
- Native signal directories are absent and no native signals are being applied.
- Most local repos are missing provider hook wiring.
- Hook command failures are visible in pane previews but not translated into a
  clear operator-facing diagnosis.

## Planned implementation validation

Fast gate for every phase:

```bash
mise run check
```

Phase-specific validation:

- Doctor engine
  - unit tests for machine checks, repo checks, and finding synthesis
  - JSON snapshot tests for stable machine-readable output
- Fix/scaffold flows
  - tempdir tests that verify additive writes and non-destructive behavior
  - failure-path tests for existing conflicting files
- Runtime/TUI diagnostics
  - bootstrap tests for operator-alert generation from doctor/runtime state
  - render tests for compatibility explanations and fix hints
- End-to-end
  - tmux integration test with missing hook files and empty native dirs
  - tmux integration test with hook-command failure preview text
  - strict real native E2E rerun before closing if hook behavior changes

## Implementation validation - 2026-04-13

Commands run:

```bash
cargo check
cargo fmt
cargo test --test cli_config -- --nocapture
cargo test runtime_findings_detect_hook_error_preview -- --nocapture
cargo test render_surfaces_runtime_setup_diagnostics_for_selected_pane -- --nocapture
mise run check
mise run verify
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
target/debug/foreman --doctor --doctor-repo . --doctor-fix --doctor-dry-run
```

Observed results:

- `cargo check` passed after doctor model, CLI plumbing, runtime synthesis, and
  render integration.
- Focused tests passed for:
  - CLI doctor output and strict failure handling
  - runtime hook-error finding synthesis
  - render-path setup diagnostics
- `mise run check` passed, including the full `cargo test --all-features` fast
  gate used by the repo.
- `mise run verify` passed end to end:
  - integration tests
  - Docker image build
  - release-confidence gauntlet
  - UX GIF/screenshot refresh
  - navigation/performance smoke
- Strict native closeout passed with all three real harness lanes enabled:
  - Claude real E2E
  - Codex real E2E
  - Pi real E2E
- The direct `--doctor --doctor-fix --doctor-dry-run` sanity run succeeded and
  produced the expected diagnosis/fix plan for the current machine state.

## CLI ergonomics follow-up - 2026-04-13

Commands run:

```bash
cargo check
cargo run -- --help
cargo run -- --setup --dry-run
cargo test --test cli_config -- --nocapture
cargo test render_surfaces_runtime_setup_diagnostics_for_selected_pane -- --nocapture
cargo test setup_returns_setup_outcome_and_writes_safe_repo_files -- --nocapture
cargo fmt
mise run check
mise run verify
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

Observed results:

- The root help output now surfaces the intended first-run path:
  `foreman --setup`, then `foreman`.
- `cargo run -- --setup --dry-run` succeeded and showed the planned config,
  Codex, Pi, and manual Claude steps for the current repo.
- Focused CLI and render tests passed, including:
  - help output expectations
  - repo inference for doctor JSON
  - setup dry-run behavior
  - setup write behavior
  - context-panel setup hint rendering
- `mise run check` passed.
- `mise run verify` passed through the full heavy suite.
- Strict native closeout passed with real Claude, Codex, and Pi E2Es enabled.

## Final scoped setup closeout - 2026-04-15

Commands run:

```bash
date '+%Y-%m-%d %H:%M %Z'
cargo test --test cli_config -- --nocapture
mise run check
mise run native-preflight
mise run verify
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
mise run install-local
foreman --setup --user --project
foreman --doctor
```

Observed results:

- `cargo test --test cli_config -- --nocapture` passed after tightening the
  scoped setup expectations and isolating `HOME` where doctor/setup tests would
  otherwise see real user-level wiring.
- `mise run check` passed on the final tree, including the full
  `cargo test --all-features` fast gate.
- `mise run native-preflight` passed on the final tree.
- `mise run verify` passed on the final tree, including:
  - fast gate
  - integration tests
  - Docker build
  - release gauntlet
  - navigation/performance smoke
  - VHS UX artifact refresh
- Strict native closeout passed on the final tree with all three real harness
  lanes enabled:
  - Claude real E2E
  - Codex real E2E
  - Pi real E2E
- `mise run install-local` updated the PATH-resolved Foreman install from this
  checkout.
- `foreman --setup --user --project` succeeded and now:
  - initializes the default config
  - creates the native signal directories
  - converges user-level and repo-level Claude, Codex, and Pi wiring
- `foreman --doctor` is now clean for machine, config, and this repo's setup.
  Remaining runtime findings are limited to already-running panes in other repos
  that still need repo wiring or a restart.
