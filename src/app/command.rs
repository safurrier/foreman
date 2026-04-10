use crate::app::{Focus, Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    StartInput,
    Select,
    Cancel,
    FocusSidebar,
    FocusPreview,
    FocusInput,
    FocusSelectedPane,
    RequestKill,
    InsertChar(char),
    Backspace,
    InsertNewline,
    SubmitDraft,
    Confirm,
    ToggleHelp,
    Search,
    FlashNavigate,
    FlashNavigateFocus,
    TogglePullRequestDetail,
    OpenPullRequest,
    CopyPullRequestUrl,
    ToggleNotificationsMuted,
    CycleNotificationProfile,
    RenameWindow,
    SpawnAgent,
    ToggleNonAgentSessions,
    ToggleNonAgentPanes,
    CycleSortMode,
    CycleTheme,
    Quit,
}

pub fn map_key_event(key: KeyEvent, focus: Focus, mode: Mode) -> Option<Command> {
    if key.kind == KeyEventKind::Release {
        return None;
    }

    if key.code == KeyCode::Esc {
        return Some(Command::Cancel);
    }

    if let Some(command) = map_modal_key_event(key, mode) {
        return Some(command);
    }

    match (mode, focus, key.code, key.modifiers) {
        (_, _, KeyCode::BackTab, _) => Some(Command::NavigateLeft),
        (_, _, KeyCode::Tab, _) => Some(Command::NavigateRight),
        (_, _, KeyCode::Char('?'), _) => Some(Command::ToggleHelp),
        (_, _, KeyCode::Char('q'), modifiers) if modifiers.is_empty() => Some(Command::Quit),
        (_, _, KeyCode::Char('1'), modifiers) if modifiers.is_empty() => {
            Some(Command::FocusSidebar)
        }
        (_, _, KeyCode::Char('2'), modifiers) if modifiers.is_empty() => {
            Some(Command::FocusPreview)
        }
        (_, _, KeyCode::Char('3'), modifiers) if modifiers.is_empty() => Some(Command::FocusInput),
        (_, _, KeyCode::Up, _) | (_, _, KeyCode::Char('k'), KeyModifiers::NONE) => {
            Some(Command::NavigateUp)
        }
        (_, _, KeyCode::Down, _) | (_, _, KeyCode::Char('j'), KeyModifiers::NONE) => {
            Some(Command::NavigateDown)
        }
        (Mode::Normal, Focus::Input, KeyCode::Enter, _) => Some(Command::StartInput),
        (Mode::Normal, _, KeyCode::Char('i'), KeyModifiers::NONE) => Some(Command::StartInput),
        (Mode::Normal, _, KeyCode::Enter, _) => Some(Command::Select),
        (Mode::Normal, _, KeyCode::Char('f'), KeyModifiers::NONE) => {
            Some(Command::FocusSelectedPane)
        }
        (Mode::Normal, _, KeyCode::Char('x'), KeyModifiers::NONE) => Some(Command::RequestKill),
        (Mode::Normal, _, KeyCode::Char('/'), KeyModifiers::NONE) => Some(Command::Search),
        (Mode::Normal, _, KeyCode::Char('s'), KeyModifiers::NONE) => Some(Command::FlashNavigate),
        (Mode::Normal, _, KeyCode::Char('S'), KeyModifiers::SHIFT) => {
            Some(Command::FlashNavigateFocus)
        }
        (Mode::Normal, _, KeyCode::Char('p'), KeyModifiers::NONE) => {
            Some(Command::TogglePullRequestDetail)
        }
        (Mode::Normal, _, KeyCode::Char('m'), KeyModifiers::NONE) => {
            Some(Command::ToggleNotificationsMuted)
        }
        (Mode::Normal, _, KeyCode::Char('n'), KeyModifiers::NONE) => {
            Some(Command::CycleNotificationProfile)
        }
        (Mode::Normal, _, KeyCode::Char('O'), KeyModifiers::SHIFT) => {
            Some(Command::OpenPullRequest)
        }
        (Mode::Normal, _, KeyCode::Char('Y'), KeyModifiers::SHIFT) => {
            Some(Command::CopyPullRequestUrl)
        }
        (Mode::Normal, _, KeyCode::Char('R'), KeyModifiers::SHIFT) => Some(Command::RenameWindow),
        (Mode::Normal, _, KeyCode::Char('N'), KeyModifiers::SHIFT) => Some(Command::SpawnAgent),
        (Mode::Normal, _, KeyCode::Char('H'), KeyModifiers::SHIFT) => {
            Some(Command::ToggleNonAgentSessions)
        }
        (Mode::Normal, _, KeyCode::Char('P'), KeyModifiers::SHIFT) => {
            Some(Command::ToggleNonAgentPanes)
        }
        (Mode::Normal, _, KeyCode::Char('o'), KeyModifiers::NONE) => Some(Command::CycleSortMode),
        (Mode::Normal, _, KeyCode::Char('t'), KeyModifiers::NONE) => Some(Command::CycleTheme),
        _ => None,
    }
}

fn map_modal_key_event(key: KeyEvent, mode: Mode) -> Option<Command> {
    match mode {
        Mode::Input => match (key.code, key.modifiers) {
            (KeyCode::Char('j'), KeyModifiers::CONTROL) | (KeyCode::Enter, KeyModifiers::SHIFT) => {
                Some(Command::InsertNewline)
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) | (KeyCode::Enter, _) => {
                Some(Command::SubmitDraft)
            }
            (KeyCode::Backspace, _) => Some(Command::Backspace),
            (KeyCode::Char(ch), modifiers) if is_text_input_modifiers(modifiers) => {
                Some(Command::InsertChar(ch))
            }
            _ => None,
        },
        Mode::Rename | Mode::Spawn => match (key.code, key.modifiers) {
            (KeyCode::Char('s'), KeyModifiers::CONTROL) | (KeyCode::Enter, _) => {
                Some(Command::SubmitDraft)
            }
            (KeyCode::Backspace, _) => Some(Command::Backspace),
            (KeyCode::Char(ch), modifiers) if is_text_input_modifiers(modifiers) => {
                Some(Command::InsertChar(ch))
            }
            _ => None,
        },
        Mode::Search => match (key.code, key.modifiers) {
            (KeyCode::Enter, _) => Some(Command::Select),
            (KeyCode::Backspace, _) => Some(Command::Backspace),
            (KeyCode::Char(ch), modifiers) if is_text_input_modifiers(modifiers) => {
                Some(Command::InsertChar(ch))
            }
            _ => None,
        },
        Mode::FlashNavigate => match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) => Some(Command::Backspace),
            (KeyCode::Char(ch), modifiers) if is_text_input_modifiers(modifiers) => {
                Some(Command::InsertChar(ch))
            }
            _ => None,
        },
        Mode::ConfirmKill => match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => Some(Command::Confirm),
            KeyCode::Char('n') | KeyCode::Char('N') => Some(Command::Cancel),
            _ => None,
        },
        Mode::Normal | Mode::PreviewScroll | Mode::Help => None,
    }
}

fn is_text_input_modifiers(modifiers: KeyModifiers) -> bool {
    modifiers.is_empty() || modifiers == KeyModifiers::SHIFT
}

#[cfg(test)]
mod tests {
    use super::{map_key_event, Command};
    use crate::app::{Focus, Mode};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn tab_cycles_focus_between_panels() {
        assert_eq!(
            map_key_event(key(KeyCode::Tab), Focus::Sidebar, Mode::Normal),
            Some(Command::NavigateRight)
        );
        assert_eq!(
            map_key_event(key(KeyCode::BackTab), Focus::Preview, Mode::Normal),
            Some(Command::NavigateLeft)
        );
    }

    #[test]
    fn enter_and_f_map_to_selection_actions_in_normal_mode() {
        assert_eq!(
            map_key_event(key(KeyCode::Enter), Focus::Sidebar, Mode::Normal),
            Some(Command::Select)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('f')), Focus::Sidebar, Mode::Normal),
            Some(Command::FocusSelectedPane)
        );
    }

    #[test]
    fn input_mode_does_not_steal_regular_typing() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('q')), Focus::Input, Mode::Input),
            Some(Command::InsertChar('q'))
        );
        assert_eq!(
            map_key_event(key(KeyCode::Esc), Focus::Input, Mode::Input),
            Some(Command::Cancel)
        );
    }

    #[test]
    fn input_mode_maps_editing_and_submit_keys() {
        assert_eq!(
            map_key_event(key(KeyCode::Enter), Focus::Input, Mode::Input),
            Some(Command::SubmitDraft)
        );
        assert_eq!(
            map_key_event(
                KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
                Focus::Input,
                Mode::Input
            ),
            Some(Command::SubmitDraft)
        );
        assert_eq!(
            map_key_event(
                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL),
                Focus::Input,
                Mode::Input
            ),
            Some(Command::InsertNewline)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Backspace), Focus::Input, Mode::Input),
            Some(Command::Backspace)
        );
    }

    #[test]
    fn rename_and_spawn_modes_treat_enter_as_submit() {
        assert_eq!(
            map_key_event(key(KeyCode::Enter), Focus::Sidebar, Mode::Rename),
            Some(Command::SubmitDraft)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Enter), Focus::Sidebar, Mode::Spawn),
            Some(Command::SubmitDraft)
        );
    }

    #[test]
    fn normal_mode_exposes_input_and_kill_shortcuts() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('i')), Focus::Sidebar, Mode::Normal),
            Some(Command::StartInput)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('x')), Focus::Sidebar, Mode::Normal),
            Some(Command::RequestKill)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Enter), Focus::Input, Mode::Normal),
            Some(Command::StartInput)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('t')), Focus::Sidebar, Mode::Normal),
            Some(Command::CycleTheme)
        );
    }

    #[test]
    fn confirm_kill_mode_accepts_yes_and_no() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('y')), Focus::Sidebar, Mode::ConfirmKill),
            Some(Command::Confirm)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('n')), Focus::Sidebar, Mode::ConfirmKill),
            Some(Command::Cancel)
        );
    }

    #[test]
    fn search_mode_captures_text_and_enter() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('q')), Focus::Sidebar, Mode::Search),
            Some(Command::InsertChar('q'))
        );
        assert_eq!(
            map_key_event(key(KeyCode::Enter), Focus::Sidebar, Mode::Search),
            Some(Command::Select)
        );
    }

    #[test]
    fn normal_mode_exposes_both_flash_variants() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('s')), Focus::Sidebar, Mode::Normal),
            Some(Command::FlashNavigate)
        );
        assert_eq!(
            map_key_event(
                KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT),
                Focus::Sidebar,
                Mode::Normal
            ),
            Some(Command::FlashNavigateFocus)
        );
    }

    #[test]
    fn normal_mode_exposes_pull_request_shortcuts() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('p')), Focus::Sidebar, Mode::Normal),
            Some(Command::TogglePullRequestDetail)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('m')), Focus::Sidebar, Mode::Normal),
            Some(Command::ToggleNotificationsMuted)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('n')), Focus::Sidebar, Mode::Normal),
            Some(Command::CycleNotificationProfile)
        );
        assert_eq!(
            map_key_event(
                KeyEvent::new(KeyCode::Char('O'), KeyModifiers::SHIFT),
                Focus::Sidebar,
                Mode::Normal
            ),
            Some(Command::OpenPullRequest)
        );
        assert_eq!(
            map_key_event(
                KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::SHIFT),
                Focus::Sidebar,
                Mode::Normal
            ),
            Some(Command::CopyPullRequestUrl)
        );
    }
}
