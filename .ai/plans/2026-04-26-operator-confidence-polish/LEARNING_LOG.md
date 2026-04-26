# Learning Log

## 2026-04-26 — Diagnosis

Live doctor showed Claude hook binaries and repo wiring were available, but only one stale Claude native signal file existed. The visible `claude.exe` panes were in compatibility mode and classified as `ERROR`. Captured pane scrollback showed older `hook error`, `Failed`, `Traceback`, or task errors above current prompt/ready lines. Root cause: Claude compatibility status scans the full captured preview for generic error terms, so stale scrollback dominates current status.

## 2026-04-26 — Completion

What matched the plan: the root Claude issue was compatibility parsing/diagnostics using too much scrollback, not broken hook wiring. Recent prompt markers now prevent stale hook/tool failures from keeping Claude panes in an error-looking state.

What diverged: the first activity/recent-event UI pass consumed vertical budget in the release gauntlet. The final design keeps activity on the existing View line and avoids duplicating long operator alerts in Recent.

One-shot improvement: any main-pane line added above selection details should be tested against the release gauntlet viewport immediately, because wrapping can hide key affordances even when unit render tests pass.
