---
id: plan-traceability
title: SPEC Traceability
description: >
  Cross-check from the root spec to planned chunks, architecture rules, and the
  intended validation layer.
---

# SPEC Traceability - spec-driven-tdd

## Coverage checklist

- [ ] **CLI, config, and logging bootstrap**
  Root spec: R1-R3, A1-A2
  Planned chunks: 2, 11, 12
  Dominant validation: CLI tests, shell smoke, `mise run check`

- [x] **tmux inventory and visibility controls**
  Root spec: R4, R7, A3, A6
  Planned chunks: 4, 5
  Dominant validation: adapter contract tests, reducer tests, and real tmux
  fixture smoke/E2E tests

- [x] **Supported harness detection and status model**
  Root spec: R5-R8, A4-A7, A11
  Planned chunks: 6, 7
  Dominant validation: status-engine unit tests, fake capture/native-signal
  contract tests

- [x] **Main UI surfaces and keyboard-first behavior**
  Root spec: R9-R11, A8-A9
  Planned chunks: 3, 5
  Dominant validation: Ratatui buffer tests, command mapping tests, focused
  smoke tests

- [ ] **Direct input and pane actions**
  Root spec: R12-R13, A10, A12
  Planned chunks: 8
  Dominant validation: reducer tests for modal state, tmux adapter contract
  tests, real tmux smoke/E2E tests

- [ ] **Search, flash navigation, and sorting**
  Root spec: R14-R16, A13-A15
  Planned chunks: 9
  Dominant validation: reducer tests, buffer tests for overlays,
  logical-selection tests

- [ ] **Pull request awareness**
  Root spec: R17, A16-A17
  Planned chunks: 10
  Dominant validation: provider contract tests, reducer tests,
  graceful-degradation smoke tests

- [ ] **Notifications**
  Root spec: R18, A18
  Planned chunks: 11
  Dominant validation: policy unit tests, backend contract tests, end-to-end
  suppression checks

- [ ] **Quality gate and definition of done**
  Root spec: R19, A19, Definition of done
  Planned chunks: 12
  Dominant validation: `cargo fmt`, `cargo clippy`, `cargo test`,
  `mise run check`, `mise run verify`

## Architecture review checklist

- [ ] `Command -> Action -> Reducer -> Effects -> Render` remains the control flow
- [ ] Render stays pure and side-effect free
- [ ] Only adapters and services talk to tmux, Git, browser, clipboard, or
  notification backends
- [ ] Every new feature names its dominant test layer before implementation
  starts
- [ ] State invariants land in tests before live integration wiring
- [ ] Native-over-compatibility precedence is enforced in one place
- [ ] Selection stability is validated for refresh, filter, sort, and collapse
  transitions
- [ ] Status does not rely on color alone
- [ ] Destructive actions require explicit confirmation
- [ ] Contract changes flow back into `SPEC.md` and `docs/architecture.md`
