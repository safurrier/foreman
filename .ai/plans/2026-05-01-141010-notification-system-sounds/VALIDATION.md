---
id: plan-validation
title: Validation Log
description: >
  How changes were verified. Append entries after testing.
  Link to artifacts — don't store them here. See _example/ for a reference.
---

# Validation

## Automated

- `cargo fmt --check` — passed.
- `cargo test -q notification_sound_source_resolves_user_sound_names -- --nocapture` — passed.
- `cargo test -q random_sound_sources_advance_selection_salt -- --nocapture` — passed.
- `cargo test -q alerter_system_sound_uses_notification_sound_without_afplay -- --nocapture` — passed.
- `cargo test -q sound_selector_resolves_audio_directories_sequentially -- --nocapture` — passed.
- `mise run check` — passed.
- `mise run install-local` — passed.

## Manual macOS Sound Smoke

- Copied:
  - `~/.config/tmuxcc/sounds/completed/reallygood.aiff` to `~/Library/Sounds/foreman-completed-reallygood.aiff`
  - `~/.config/tmuxcc/sounds/completed/sobad.aiff` to `~/Library/Sounds/foreman-completed-sobad.aiff`
  - `~/.config/tmuxcc/sounds/needs_input/askingqs.aiff` to `~/Library/Sounds/foreman-needs-input-askingqs.aiff`
- Ran `alerter --sound foreman-completed-reallygood` with Focus off. User confirmed the custom sound was audible.
- Ran the same command with Focus / Do Not Disturb on. Notification delivered, and user replied "I did not" when asked whether the sound was audible.
