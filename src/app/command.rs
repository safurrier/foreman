use crate::app::{Focus, Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Select,
    Cancel,
    FocusSidebar,
    FocusPreview,
    FocusInput,
    FocusSelectedPane,
    ToggleHelp,
    Search,
    FlashNavigate,
    RenameWindow,
    SpawnAgent,
    ToggleNonAgentSessions,
    ToggleNonAgentPanes,
    CycleSortMode,
    Quit,
}

pub fn map_key_event(key: KeyEvent, focus: Focus, mode: Mode) -> Option<Command> {
    if key.kind == KeyEventKind::Release {
        return None;
    }

    match (mode, focus, key.code, key.modifiers) {
        (_, _, KeyCode::Esc, _) => Some(Command::Cancel),
        (_, _, KeyCode::BackTab, _) => Some(Command::NavigateLeft),
        (_, _, KeyCode::Tab, _) => Some(Command::NavigateRight),
        (Mode::Input, Focus::Input, _, _) => None,
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
        (Mode::Normal, _, KeyCode::Enter, _) => Some(Command::Select),
        (Mode::Normal, _, KeyCode::Char('f'), KeyModifiers::NONE) => {
            Some(Command::FocusSelectedPane)
        }
        (Mode::Normal, _, KeyCode::Char('/'), KeyModifiers::NONE) => Some(Command::Search),
        (Mode::Normal, _, KeyCode::Char('s'), KeyModifiers::NONE) => Some(Command::FlashNavigate),
        (Mode::Normal, _, KeyCode::Char('R'), KeyModifiers::SHIFT) => Some(Command::RenameWindow),
        (Mode::Normal, _, KeyCode::Char('N'), KeyModifiers::SHIFT) => Some(Command::SpawnAgent),
        (Mode::Normal, _, KeyCode::Char('H'), KeyModifiers::SHIFT) => {
            Some(Command::ToggleNonAgentSessions)
        }
        (Mode::Normal, _, KeyCode::Char('P'), KeyModifiers::SHIFT) => {
            Some(Command::ToggleNonAgentPanes)
        }
        (Mode::Normal, _, KeyCode::Char('o'), KeyModifiers::NONE) => Some(Command::CycleSortMode),
        _ => None,
    }
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
            None
        );
        assert_eq!(
            map_key_event(key(KeyCode::Esc), Focus::Input, Mode::Input),
            Some(Command::Cancel)
        );
    }
}
