---
id: plan-learning-log
title: Learning Log
description: >
  Dev diary. Append timestamped entries for problems, adaptations,
  user feedback, and surprises. See _example/ for a reference.
---

# Learning Log

## 2026-04-10 09:15 - CI already existed, but the real 1.0 gap was CD

The repo already had `ci.yml`, so the problem was not "set up GitHub Actions"
from scratch. The real gap was that there was no release workflow at all and
the crate was still versioned `0.1.0`. That changed the task from CI setup to a
release-automation pass with versioning and packaging.

## 2026-04-10 09:20 - Release bundles need the hook binaries too

It would have been easy to ship only `foreman`, but that would make native
Claude, Codex, and Pi support incomplete the moment someone downloaded a
release archive. The release package needs to include `foreman-claude-hook`,
`foreman-codex-hook`, and `foreman-pi-hook` as first-class release artifacts.

## 2026-04-10 10:05 - Actionlint caught a real runner-label drift

The first release workflow draft used `macos-13` for the Intel macOS build.
`actionlint` flagged that as an outdated label in its current runner catalog, so
the workflow moved to `macos-15-intel`. That was a good reminder that workflow
syntax review is not enough; the runner catalog changes underneath you.

## 2026-04-10 10:25 - The 1.0 version bump exposed a Docker cache weakness

`mise run verify` broke after the crate version moved to `1.0.0`, not because
the app stopped building, but because the Dockerfile tied the dependency-cache
layer directly to `Cargo.toml`. A version bump invalidated that layer, forced a
fresh dependency build inside Docker, and exposed unreliable network resolution
to `crates.io` in this Colima environment.

The fix was to add a stable `Cargo.docker.toml` dependency manifest for the
cache-priming step and then copy the real manifest and source later. That made
the heavy gate stable again and also let the container image preserve the real
`foreman` command name plus the hook binaries.

## 2026-04-10 10:30 - PR creation is blocked only by repo plumbing

The product and release automation work are ready for a PR, but this checkout
has no git remote configured. That means push and `gh pr create` are blocked by
repository plumbing, not by code or validation state.

## 2026-04-10 10:55 - The tmux E2E harness had a hidden shell portability bug

The first GitHub Actions run failed even though local validation was green.
The app was fine; the failure was in the tmux test fixture and a few runtime
tests that assumed `zsh` existed. On `ubuntu-latest`, those panes exited
immediately, the tmux server shut down, and the tests reported "no server
running".

The fix was to make the fixture and the non-ignored tmux E2E commands use
portable `sh` plus `printf`, and to tighten the quoting in the helper that
keeps dashboard sessions alive. That made the same test layer work on both
macOS and Linux instead of only on the local machine.

## 2026-04-10 11:25 - tmux window indexes were another hidden local assumption

The next GitHub Actions run found a second portability issue in the same test
layer. Several runtime and tmux smoke tests hardcoded `alpha:1` as if the first
window index were stable. That was true locally, but not on the Linux runner.

The fix was to stop targeting inferred window numbers and instead split panes
from the real pane id returned by the fixture. Assertions that only needed the
single test window now target the session name rather than a guessed `:1`
window index.

## 2026-04-10 11:45 - full-gate confirmation mattered more than the flaky pre-commit signal

The failed pre-commit run looked like a new notification regression, but the
test passed when rerun directly and again inside the full `mise run check`
gate. That suggests the actionable issue was still the tmux portability fix,
not the notification logic.

The useful rule here is to treat a one-off failure in the tmux-heavy suite as a
signal to rerun the exact test and then the full fast gate before changing
behavioral code. That keeps the response proportional and avoids papering over
timing noise with unnecessary product changes.

## 2026-04-10 12:10 - heavy CI exposed a transient tmux startup race

The next failing CI leg was not another shell-compatibility bug or a product
regression. `CI/Full Validation` failed in `claude_native` because the very
first tmux `new-session` call sometimes hit `server exited unexpectedly` under
the heavier verification load, while immediate reruns passed.

The durable fix was to harden the shared tmux fixture, not the app code. The
fixture now retries transient tmux startup errors during session creation and
pane splits. That stabilized the same test path locally, including repeated
reruns of the previously flaky Claude-native bootstrap and a full
`mise run verify`.
