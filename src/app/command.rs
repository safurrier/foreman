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
    PreviewScrollUp,
    PreviewScrollDown,
    PreviewPageUp,
    PreviewPageDown,
    PreviewTop,
    PreviewBottom,
    HelpScrollUp,
    HelpScrollDown,
    HelpPageUp,
    HelpPageDown,
    HelpTop,
    HelpBottom,
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
    CycleHarnessFilter,
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
        (Mode::Normal | Mode::PreviewScroll, Focus::Preview, KeyCode::Up, _)
        | (
            Mode::Normal | Mode::PreviewScroll,
            Focus::Preview,
            KeyCode::Char('k'),
            KeyModifiers::NONE,
        ) => Some(Command::PreviewScrollUp),
        (Mode::Normal | Mode::PreviewScroll, Focus::Preview, KeyCode::Down, _)
        | (
            Mode::Normal | Mode::PreviewScroll,
            Focus::Preview,
            KeyCode::Char('j'),
            KeyModifiers::NONE,
        ) => Some(Command::PreviewScrollDown),
        (Mode::Normal | Mode::PreviewScroll, Focus::Preview, KeyCode::PageUp, _) => {
            Some(Command::PreviewPageUp)
        }
        (Mode::Normal | Mode::PreviewScroll, Focus::Preview, KeyCode::PageDown, _) => {
            Some(Command::PreviewPageDown)
        }
        (Mode::Normal | Mode::PreviewScroll, Focus::Preview, KeyCode::Home, _) => {
            Some(Command::PreviewTop)
        }
        (Mode::Normal | Mode::PreviewScroll, Focus::Preview, KeyCode::End, _) => {
            Some(Command::PreviewBottom)
        }
        (_, _, KeyCode::Up, _) | (_, _, KeyCode::Char('k'), KeyModifiers::NONE) => {
            Some(Command::NavigateUp)
        }
        (_, _, KeyCode::Down, _) | (_, _, KeyCode::Char('j'), KeyModifiers::NONE) => {
            Some(Command::NavigateDown)
        }
        (Mode::Normal | Mode::PreviewScroll, Focus::Input, KeyCode::Enter, _) => {
            Some(Command::StartInput)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('i'), KeyModifiers::NONE) => {
            Some(Command::StartInput)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Enter, _) => Some(Command::Select),
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('f'), KeyModifiers::NONE) => {
            Some(Command::FocusSelectedPane)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('x'), KeyModifiers::NONE) => {
            Some(Command::RequestKill)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('/'), KeyModifiers::NONE) => {
            Some(Command::Search)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('s'), KeyModifiers::NONE) => {
            Some(Command::FlashNavigate)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('S'), _) => {
            Some(Command::FlashNavigateFocus)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('p'), KeyModifiers::NONE) => {
            Some(Command::TogglePullRequestDetail)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('m'), KeyModifiers::NONE) => {
            Some(Command::ToggleNotificationsMuted)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('n'), KeyModifiers::NONE) => {
            Some(Command::CycleNotificationProfile)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('O'), _) => {
            Some(Command::OpenPullRequest)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('Y'), _) => {
            Some(Command::CopyPullRequestUrl)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('R'), _) => {
            Some(Command::RenameWindow)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('N'), _) => Some(Command::SpawnAgent),
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('H'), _) => {
            Some(Command::ToggleNonAgentSessions)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('P'), _) => {
            Some(Command::ToggleNonAgentPanes)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('h'), KeyModifiers::NONE) => {
            Some(Command::CycleHarnessFilter)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('o'), KeyModifiers::NONE) => {
            Some(Command::CycleSortMode)
        }
        (Mode::Normal | Mode::PreviewScroll, _, KeyCode::Char('t'), KeyModifiers::NONE) => {
            Some(Command::CycleTheme)
        }
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
        Mode::Help => match (key.code, key.modifiers) {
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                Some(Command::HelpScrollUp)
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                Some(Command::HelpScrollDown)
            }
            (KeyCode::PageUp, _) => Some(Command::HelpPageUp),
            (KeyCode::PageDown, _) => Some(Command::HelpPageDown),
            (KeyCode::Home, _) => Some(Command::HelpTop),
            (KeyCode::End, _) => Some(Command::HelpBottom),
            _ => None,
        },
        Mode::Normal | Mode::PreviewScroll => None,
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

    fn ctrl_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    fn shift_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::SHIFT)
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
        assert_eq!(
            map_key_event(key(KeyCode::Char('h')), Focus::Sidebar, Mode::Normal),
            Some(Command::CycleHarnessFilter)
        );
    }

    #[test]
    fn preview_focus_maps_navigation_to_preview_scroll() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('j')), Focus::Preview, Mode::PreviewScroll),
            Some(Command::PreviewScrollDown)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('k')), Focus::Preview, Mode::PreviewScroll),
            Some(Command::PreviewScrollUp)
        );
        assert_eq!(
            map_key_event(key(KeyCode::PageDown), Focus::Preview, Mode::Normal),
            Some(Command::PreviewPageDown)
        );
    }

    #[test]
    fn preview_scroll_mode_keeps_normal_shortcuts_available() {
        let cases = vec![
            (KeyCode::Char('i'), Command::StartInput),
            (KeyCode::Enter, Command::Select),
            (KeyCode::Char('f'), Command::FocusSelectedPane),
            (KeyCode::Char('/'), Command::Search),
            (KeyCode::Char('p'), Command::TogglePullRequestDetail),
            (KeyCode::Char('m'), Command::ToggleNotificationsMuted),
        ];

        for (key_code, command) in cases {
            assert_eq!(
                map_key_event(key(key_code), Focus::Preview, Mode::PreviewScroll),
                Some(command)
            );
        }
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
    fn help_mode_maps_scroll_keys_without_reusing_sidebar_navigation() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('j')), Focus::Sidebar, Mode::Help),
            Some(Command::HelpScrollDown)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('k')), Focus::Sidebar, Mode::Help),
            Some(Command::HelpScrollUp)
        );
        assert_eq!(
            map_key_event(key(KeyCode::PageDown), Focus::Sidebar, Mode::Help),
            Some(Command::HelpPageDown)
        );
        assert_eq!(
            map_key_event(key(KeyCode::PageUp), Focus::Sidebar, Mode::Help),
            Some(Command::HelpPageUp)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Home), Focus::Sidebar, Mode::Help),
            Some(Command::HelpTop)
        );
        assert_eq!(
            map_key_event(key(KeyCode::End), Focus::Sidebar, Mode::Help),
            Some(Command::HelpBottom)
        );
    }

    #[test]
    fn normal_mode_exposes_both_flash_variants() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('s')), Focus::Sidebar, Mode::Normal),
            Some(Command::FlashNavigate)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('S')), Focus::Sidebar, Mode::Normal),
            Some(Command::FlashNavigateFocus)
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
            map_key_event(key(KeyCode::Char('O')), Focus::Sidebar, Mode::Normal),
            Some(Command::OpenPullRequest)
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
            map_key_event(key(KeyCode::Char('Y')), Focus::Sidebar, Mode::Normal),
            Some(Command::CopyPullRequestUrl)
        );
        assert_eq!(
            map_key_event(
                KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::SHIFT),
                Focus::Sidebar,
                Mode::Normal
            ),
            Some(Command::CopyPullRequestUrl)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('N')), Focus::Sidebar, Mode::Normal),
            Some(Command::SpawnAgent)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('R')), Focus::Sidebar, Mode::Normal),
            Some(Command::RenameWindow)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('H')), Focus::Sidebar, Mode::Normal),
            Some(Command::ToggleNonAgentSessions)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('P')), Focus::Sidebar, Mode::Normal),
            Some(Command::ToggleNonAgentPanes)
        );
    }

    #[test]
    fn advertised_normal_mode_keybinds_map_to_expected_commands() {
        let cases = vec![
            (
                key(KeyCode::Esc),
                Focus::Sidebar,
                Mode::Normal,
                Command::Cancel,
            ),
            (
                key(KeyCode::Tab),
                Focus::Sidebar,
                Mode::Normal,
                Command::NavigateRight,
            ),
            (
                key(KeyCode::BackTab),
                Focus::Preview,
                Mode::Normal,
                Command::NavigateLeft,
            ),
            (
                key(KeyCode::Char('?')),
                Focus::Sidebar,
                Mode::Normal,
                Command::ToggleHelp,
            ),
            (
                key(KeyCode::Char('q')),
                Focus::Sidebar,
                Mode::Normal,
                Command::Quit,
            ),
            (
                key(KeyCode::Char('1')),
                Focus::Preview,
                Mode::Normal,
                Command::FocusSidebar,
            ),
            (
                key(KeyCode::Char('2')),
                Focus::Sidebar,
                Mode::Normal,
                Command::FocusPreview,
            ),
            (
                key(KeyCode::Char('3')),
                Focus::Sidebar,
                Mode::Normal,
                Command::FocusInput,
            ),
            (
                key(KeyCode::Up),
                Focus::Sidebar,
                Mode::Normal,
                Command::NavigateUp,
            ),
            (
                key(KeyCode::Down),
                Focus::Sidebar,
                Mode::Normal,
                Command::NavigateDown,
            ),
            (
                key(KeyCode::Char('k')),
                Focus::Sidebar,
                Mode::Normal,
                Command::NavigateUp,
            ),
            (
                key(KeyCode::Char('j')),
                Focus::Sidebar,
                Mode::Normal,
                Command::NavigateDown,
            ),
            (
                key(KeyCode::Char('i')),
                Focus::Sidebar,
                Mode::Normal,
                Command::StartInput,
            ),
            (
                key(KeyCode::Enter),
                Focus::Sidebar,
                Mode::Normal,
                Command::Select,
            ),
            (
                key(KeyCode::Enter),
                Focus::Input,
                Mode::Normal,
                Command::StartInput,
            ),
            (
                key(KeyCode::Char('f')),
                Focus::Sidebar,
                Mode::Normal,
                Command::FocusSelectedPane,
            ),
            (
                key(KeyCode::Char('x')),
                Focus::Sidebar,
                Mode::Normal,
                Command::RequestKill,
            ),
            (
                key(KeyCode::Char('/')),
                Focus::Sidebar,
                Mode::Normal,
                Command::Search,
            ),
            (
                key(KeyCode::Char('s')),
                Focus::Sidebar,
                Mode::Normal,
                Command::FlashNavigate,
            ),
            (
                key(KeyCode::Char('S')),
                Focus::Sidebar,
                Mode::Normal,
                Command::FlashNavigateFocus,
            ),
            (
                shift_key(KeyCode::Char('S')),
                Focus::Sidebar,
                Mode::Normal,
                Command::FlashNavigateFocus,
            ),
            (
                key(KeyCode::Char('p')),
                Focus::Sidebar,
                Mode::Normal,
                Command::TogglePullRequestDetail,
            ),
            (
                key(KeyCode::Char('m')),
                Focus::Sidebar,
                Mode::Normal,
                Command::ToggleNotificationsMuted,
            ),
            (
                key(KeyCode::Char('n')),
                Focus::Sidebar,
                Mode::Normal,
                Command::CycleNotificationProfile,
            ),
            (
                key(KeyCode::Char('O')),
                Focus::Sidebar,
                Mode::Normal,
                Command::OpenPullRequest,
            ),
            (
                shift_key(KeyCode::Char('O')),
                Focus::Sidebar,
                Mode::Normal,
                Command::OpenPullRequest,
            ),
            (
                key(KeyCode::Char('Y')),
                Focus::Sidebar,
                Mode::Normal,
                Command::CopyPullRequestUrl,
            ),
            (
                shift_key(KeyCode::Char('Y')),
                Focus::Sidebar,
                Mode::Normal,
                Command::CopyPullRequestUrl,
            ),
            (
                key(KeyCode::Char('R')),
                Focus::Sidebar,
                Mode::Normal,
                Command::RenameWindow,
            ),
            (
                key(KeyCode::Char('N')),
                Focus::Sidebar,
                Mode::Normal,
                Command::SpawnAgent,
            ),
            (
                key(KeyCode::Char('h')),
                Focus::Sidebar,
                Mode::Normal,
                Command::CycleHarnessFilter,
            ),
            (
                key(KeyCode::Char('H')),
                Focus::Sidebar,
                Mode::Normal,
                Command::ToggleNonAgentSessions,
            ),
            (
                key(KeyCode::Char('P')),
                Focus::Sidebar,
                Mode::Normal,
                Command::ToggleNonAgentPanes,
            ),
            (
                key(KeyCode::Char('o')),
                Focus::Sidebar,
                Mode::Normal,
                Command::CycleSortMode,
            ),
            (
                key(KeyCode::Char('t')),
                Focus::Sidebar,
                Mode::Normal,
                Command::CycleTheme,
            ),
        ];

        for (key_event, focus, mode, expected) in cases {
            assert_eq!(
                map_key_event(key_event, focus, mode),
                Some(expected),
                "expected {expected:?} for {key_event:?} in {mode:?}/{focus:?}"
            );
        }
    }

    #[test]
    fn advertised_modal_keybinds_map_to_expected_commands() {
        let cases = vec![
            (
                key(KeyCode::Esc),
                Focus::Input,
                Mode::Input,
                Command::Cancel,
            ),
            (
                key(KeyCode::Enter),
                Focus::Input,
                Mode::Input,
                Command::SubmitDraft,
            ),
            (
                ctrl_key(KeyCode::Char('s')),
                Focus::Input,
                Mode::Input,
                Command::SubmitDraft,
            ),
            (
                ctrl_key(KeyCode::Char('j')),
                Focus::Input,
                Mode::Input,
                Command::InsertNewline,
            ),
            (
                key(KeyCode::Backspace),
                Focus::Input,
                Mode::Input,
                Command::Backspace,
            ),
            (
                key(KeyCode::Enter),
                Focus::Sidebar,
                Mode::Rename,
                Command::SubmitDraft,
            ),
            (
                key(KeyCode::Enter),
                Focus::Sidebar,
                Mode::Spawn,
                Command::SubmitDraft,
            ),
            (
                key(KeyCode::Enter),
                Focus::Sidebar,
                Mode::Search,
                Command::Select,
            ),
            (
                key(KeyCode::Backspace),
                Focus::Sidebar,
                Mode::Search,
                Command::Backspace,
            ),
            (
                key(KeyCode::Backspace),
                Focus::Sidebar,
                Mode::FlashNavigate,
                Command::Backspace,
            ),
            (
                key(KeyCode::Enter),
                Focus::Sidebar,
                Mode::ConfirmKill,
                Command::Confirm,
            ),
            (
                key(KeyCode::Char('j')),
                Focus::Sidebar,
                Mode::Help,
                Command::HelpScrollDown,
            ),
            (
                key(KeyCode::Char('k')),
                Focus::Sidebar,
                Mode::Help,
                Command::HelpScrollUp,
            ),
            (
                key(KeyCode::PageDown),
                Focus::Sidebar,
                Mode::Help,
                Command::HelpPageDown,
            ),
            (
                key(KeyCode::PageUp),
                Focus::Sidebar,
                Mode::Help,
                Command::HelpPageUp,
            ),
            (
                key(KeyCode::Home),
                Focus::Sidebar,
                Mode::Help,
                Command::HelpTop,
            ),
            (
                key(KeyCode::End),
                Focus::Sidebar,
                Mode::Help,
                Command::HelpBottom,
            ),
            (
                key(KeyCode::Char('y')),
                Focus::Sidebar,
                Mode::ConfirmKill,
                Command::Confirm,
            ),
            (
                key(KeyCode::Char('n')),
                Focus::Sidebar,
                Mode::ConfirmKill,
                Command::Cancel,
            ),
        ];

        for (key_event, focus, mode, expected) in cases {
            assert_eq!(
                map_key_event(key_event, focus, mode),
                Some(expected),
                "expected {expected:?} for {key_event:?} in {mode:?}/{focus:?}"
            );
        }
    }
}
