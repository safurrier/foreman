# Validation

| Slice | Swift tests | Overlay lane | Install | Notes |
|---|---|---|---|---|
| 1 Hotkey reliability | passed | passed | pending | Carbon hotkey registration now uses the dispatcher target for both registration and handler install. |
| 2 Keyboard mode pass-through | passed | passed | pending | Flash/help modes now bypass text editing pass-through for advertised typed commands. |
| 3 UI gauntlet coverage | passed | passed | pending | Scripted gauntlet now proves Cmd+J label selection, ? help, and j help scrolling. |
| 4 Terminal activation lifecycle | passed | passed | pending | Focus uses the persistent activator instead of recreating auto activation at focus time. |
| 5 Settings hotkey status freshness | passed | passed | pending | Recorder changes use a shortcut-specific callback and Settings content refreshes with new status. |
| 6 includeAllPanes reload semantics | passed | passed | pending | includeAllPanes preference changes schedule a debounced reload and have unit coverage. |
| Final | passed | passed | passed | `mise run validate-macos-overlay-change` passed; installed and relaunched `~/Applications/Foreman.app` with persisted Ctrl+F. |
