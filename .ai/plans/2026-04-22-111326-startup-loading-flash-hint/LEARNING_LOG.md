# Learning Log — startup-loading-flash-hint

## 2026-04-22

### Initial diagnosis

- Startup still paid for the first tmux inventory before the operator saw any frame.
- The newer staged refresh improvements helped steady-state responsiveness, but bootstrap was still synchronous.
- Flash mode still used a popup hint that covered the rows it was trying to label.

### What changed

- Runtime now starts with a loading state and paints immediately.
- Initial inventory is requested asynchronously after first paint.
- Startup preview capture is lighter than steady-state refresh.
- Flash guidance moved out of a blocking popup and into inline/footer UI.

### Validation-driven corrections

- Startup-loading changed test timing enough that some smokes and real-provider E2Es had stale assumptions about when rows existed and when selection had moved away from the active task.
- The release gauntlet had to wait on loaded dashboard state instead of assuming the first frame already contained sessions.
- Claude and Pi real E2Es had to wait for both task rows, then move selection explicitly before expecting completion notifications.

### What diverged from the initial approach

- The startup/render change itself was straightforward.
- Most of the real work was in test alignment: once the product stopped blocking first paint, anything assuming synchronous inventory at startup had to be re-anchored to actual loaded-state boundaries.

### One-shot lessons

- If startup work is expensive, prefer a truthful loading frame over hiding the app until the first full snapshot is ready.
- When early rendering changes, revisit any test that encodes “startup implies loaded” as an assumption.
- Flash guidance should never cover the targets it is teaching the operator to select.
