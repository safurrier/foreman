# Learning Log

## 2026-05-05

- User reported arrow keys do not scroll/navigate after the keyboard pass-through hardening. Treating as a blocker before PR.
- Fix keeps Up/Down/PageUp/PageDown owned by overlay navigation outside compose mode. Compose still passes through all ordinary editing keys for multiline text editing.
- Scripted gauntlet now posts Down Arrow and asserts selected pane changes before focus/send proof.
