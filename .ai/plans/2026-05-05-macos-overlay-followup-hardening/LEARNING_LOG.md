# Learning Log

## 2026-05-05

- User confirmed Ctrl+F worked after the first hotkey fix, then later reported it stopped working again. Treating hotkey reliability as still unresolved and prioritizing a lower-level Carbon target alignment plus fresh install proof.
- Review found a real regression in the text-editing pass-through fix: focused search field can steal flash/help typed commands. This must be fixed before PR.
- The scripted gauntlet reproduced the issue immediately: Cmd+J worked, but `?` was inserted into search instead of opening help. The fix keeps AppKit text editing for normal search input but lets flash/help modes and list `?` win.
- The gauntlet also exposed a hidden assumption after removing double reloads: persisted/debug search text can make `entries` empty in scripted checks. The gauntlet now clears query before direct focus proof.
- Terminal activation now uses the persistent observer-backed activator on focus. Preferences can still refresh the activator, but focus no longer throws away recency state by constructing a new `AutoTerminalActivator`.
