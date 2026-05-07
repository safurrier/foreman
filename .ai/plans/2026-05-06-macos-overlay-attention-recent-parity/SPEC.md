# macOS Overlay Attention/Recent Sort Parity

## Goal
Make the macOS overlay's attention sort match Foreman's TUI `attention-recent` semantics: attention/status priority first, then most recent activity, then deterministic title/id tie-breakers.

## Scope
- Rename overlay label/help text from generic `Attention First` to `Attention → Recent` where user-facing.
- Keep persisted raw value compatibility (`attentionFirst`) so existing settings survive.
- Update sorting implementation and unit coverage.
- Validate Swift overlay tests.

## Non-goals
- Remove the overlay-only `Recent First` mode.
- Change Rust/TUI sort behavior.
