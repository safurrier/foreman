# Learning Log — capture-refresh-perf

## 2026-04-21

### Initial diagnosis

- Local lag after the recent `j/k` work is no longer dominated by reducer or render time.
- Live traces point at synchronous tmux refresh on the main loop.
- `capture-pane` across every pane on every tick is the main cost center.

### Working hypothesis

- A two-step fix is better than jumping straight to a background worker:
  1. reduce default refresh pressure
  2. stop capturing every pane every tick

### Risks to watch

- preview staleness becoming misleading
- pane reuse hiding behind cached previews
- tests depending on eager preview refresh

### What actually happened

- Lowering `capture_lines` and raising `poll_interval_ms` helped, but it did not remove the hitch by itself.
- Selected/visible-first capture with cached off-screen previews reduced the number of `capture-pane` calls, but synchronous refresh still produced enough wall-clock latency to show up as a hitch in crowded worlds.
- The final shape needed both layers:
  1. lower default refresh pressure
  2. stage preview capture
  3. move tmux inventory refresh work off the main loop

### Validation-driven corrections

- The first version of the crowded profiling smoke budgeted raw `inventory_tmux` wall time. That stopped matching user-visible performance once refresh moved into a worker. The better invariant was overlap behavior: preview reuse must be active and `move-selection` must stay bounded while refresh is happening in the background.
- The async refresh path surfaced a notification-runtime test race. The underlying product behavior was still correct, but the test used a fixed sleep and could quit before the suppressed notification decision had been logged. The test now waits for the expected `profile_filtered` log line instead of sleeping blindly.

### One-shot lessons

- If a trace shows `move-selection` and `render_frame` are already cheap, treat synchronous background work on the main loop as the prime suspect before doing more reducer or render caching.
- When moving work off the main loop, re-check any tests that rely on fixed sleeps or on synchronous side effects appearing before quit.
- For future navigation perf slices, keep the operator-facing metric tied to overlap behavior, not just raw worker time.
