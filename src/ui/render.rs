use crate::app::{AgentStatus, AppState, Focus, ModalState, Mode, SelectionTarget};
use crate::services::pull_requests::PullRequestLookup;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, state: &AppState) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, vertical[0], state);
    render_body(frame, vertical[1], state);
    render_footer(frame, vertical[2], state);

    if state.mode == Mode::Help {
        render_help(frame);
    } else if state.modal.is_some() {
        render_modal(frame, state);
    } else if state.mode == Mode::Search {
        render_search_overlay(frame, state);
    } else if state.mode == Mode::FlashNavigate {
        render_flash_overlay(frame, state);
    }
}

fn render_header(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let visible_count = state.visible_targets().len();
    let pull_request = state
        .pull_request_compact_label()
        .map(|label| format!(" | {label}"))
        .unwrap_or_default();
    let content = format!(
        "Foreman | mode={} | focus={} | visible_targets={}{}",
        state.mode_label(),
        state.focus_label(),
        visible_count,
        pull_request
    );
    let header =
        Paragraph::new(content).block(Block::default().borders(Borders::ALL).title("Header"));
    frame.render_widget(header, area);
}

fn render_body(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(34), Constraint::Percentage(66)])
        .split(area);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
        .split(columns[1]);

    render_sidebar(frame, columns[0], state);
    render_preview(frame, right[0], state);
    render_input(frame, right[1], state);
}

fn render_sidebar(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let content = if let Some(error) = &state.startup_error {
        if state.inventory.sessions.is_empty() {
            format!("Startup issue:\n{error}")
        } else {
            sidebar_lines(state)
        }
    } else {
        sidebar_lines(state)
    };

    let sidebar = Paragraph::new(content)
        .block(focused_block("Sidebar", state.focus == Focus::Sidebar))
        .wrap(Wrap { trim: false });
    frame.render_widget(sidebar, area);
}

fn render_preview(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let content = if let Some(error) = &state.startup_error {
        if state.inventory.sessions.is_empty() {
            format!("tmux unavailable or empty.\n\n{error}")
        } else {
            preview_lines(state)
        }
    } else {
        preview_lines(state)
    };

    let preview = Paragraph::new(content)
        .block(focused_block("Preview", state.focus == Focus::Preview))
        .wrap(Wrap { trim: false });
    frame.render_widget(preview, area);
}

fn render_input(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let content = if state.mode == Mode::Input || !state.input_draft.text.is_empty() {
        if state.input_draft.text.is_empty() {
            "Compose text for the selected pane.\n\nCtrl+S: send | Esc: cancel".to_string()
        } else {
            format!("{}\n\nCtrl+S: send | Esc: cancel", state.input_draft.text)
        }
    } else {
        match state.focus {
            Focus::Input => "Input focused. Press Enter or i to compose.".to_string(),
            _ => "Direct input is available for the selected pane.".to_string(),
        }
    };

    let input = Paragraph::new(content)
        .block(focused_block("Input", state.focus == Focus::Input))
        .wrap(Wrap { trim: false });
    frame.render_widget(input, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let hint = match state.mode {
        Mode::Input => "Ctrl+S Send | Esc Cancel",
        Mode::Search => "Type Filter | Enter Confirm | Esc Restore",
        Mode::FlashNavigate => "Type Label | Esc Cancel",
        Mode::Rename => "Enter Apply | Esc Cancel",
        Mode::Spawn => "Enter Spawn | Esc Cancel",
        Mode::ConfirmKill => "Enter/Y Confirm | Esc Cancel",
        Mode::Help => "? Close | q Quit",
        Mode::Normal | Mode::PreviewScroll => {
            if state.selected_pull_request().is_some() {
                "? Help | / Search | s Flash | p PR | O Open | Y Copy | q Quit"
            } else {
                "? Help | / Search | s Flash | p PR | q Quit"
            }
        }
    };
    let footer = Paragraph::new(format!(
        "MODE: {} | FOCUS: {} | {}",
        state.mode_label(),
        state.focus_label(),
        hint
    ))
    .block(Block::default().borders(Borders::ALL).title("Footer"));
    frame.render_widget(footer, area);
}

fn render_help(frame: &mut Frame<'_>) {
    let popup = centered_rect(64, 45, frame.area());
    frame.render_widget(Clear, popup);
    let help = Paragraph::new(
        "Help\n\nj/k or arrows: move\nEnter: select\nTab or focus command: move focus\ni: compose direct input\np: toggle PR detail\nShift+O: open PR in browser\nShift+Y: copy PR URL\nx: confirm kill for selected pane\nCtrl+S: submit the active draft\n?: toggle help\nq: quit",
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    )
    .wrap(Wrap { trim: false });
    frame.render_widget(help, popup);
}

fn render_search_overlay(frame: &mut Frame<'_>, state: &AppState) {
    let popup = centered_rect(56, 28, frame.area());
    frame.render_widget(Clear, popup);
    let query = state.search_query().unwrap_or("");
    let body = format!(
        "Query: {}\nMatches: {}\n\nType to filter\nEnter: confirm\nEsc: restore previous selection",
        if query.is_empty() { "<empty>" } else { query },
        state.visible_targets().len()
    );
    let overlay = Paragraph::new(body)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(overlay, popup);
}

fn render_flash_overlay(frame: &mut Frame<'_>, state: &AppState) {
    let popup = centered_rect(56, 28, frame.area());
    frame.render_widget(Clear, popup);
    let (mode_name, typed) = state
        .flash
        .as_ref()
        .map(|flash| {
            let mode_name = match flash.kind {
                crate::app::FlashNavigateKind::Jump => "Jump",
                crate::app::FlashNavigateKind::JumpAndFocus => "Jump + Focus",
            };
            let typed = if flash.draft.text.is_empty() {
                "<empty>".to_string()
            } else {
                flash.draft.text.clone()
            };
            (mode_name, typed)
        })
        .unwrap_or(("Jump", "<empty>".to_string()));
    let body = format!(
        "Mode: {}\nTyped: {}\nLabels: {}\n\nType the visible label\nEsc: cancel",
        mode_name,
        typed,
        state.flash_targets().len()
    );
    let overlay = Paragraph::new(body)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Flash")
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(overlay, popup);
}

fn render_modal(frame: &mut Frame<'_>, state: &AppState) {
    let Some(modal) = state.modal.as_ref() else {
        return;
    };

    let popup = centered_rect(62, 38, frame.area());
    frame.render_widget(Clear, popup);

    let (title, body, border_color) = match modal {
        ModalState::RenameWindow { window_id, draft } => (
            "Rename Window",
            format!(
                "Target window: {}\n\n{}\n\nEnter or Ctrl+S: apply\nEsc: cancel",
                window_id.as_str(),
                modal_draft_text(draft, "Type a new window name.")
            ),
            Color::Cyan,
        ),
        ModalState::SpawnWindow { session_id, draft } => (
            "Spawn Agent",
            format!(
                "Target session: {}\n\n{}\n\nEnter or Ctrl+S: spawn\nEsc: cancel",
                session_id.as_str(),
                modal_draft_text(draft, "Type the command to run in the new window.")
            ),
            Color::Cyan,
        ),
        ModalState::ConfirmKill { pane_id } => (
            "Confirm Kill",
            format!(
                "Kill pane {}?\n\nEnter or y: confirm\nEsc or n: cancel",
                pane_id.as_str()
            ),
            Color::Yellow,
        ),
    };

    let modal = Paragraph::new(body)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(
                    Style::default()
                        .fg(border_color)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(modal, popup);
}

fn modal_draft_text(draft: &crate::app::TextDraft, placeholder: &str) -> String {
    if draft.text.is_empty() {
        placeholder.to_string()
    } else {
        draft.text.clone()
    }
}

fn focused_block(title: &str, focused: bool) -> Block<'static> {
    let title = if focused {
        format!("* {title}")
    } else {
        title.to_string()
    };

    let style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(style)
}

fn sidebar_lines(state: &AppState) -> String {
    let visible_targets = state.visible_targets();
    if visible_targets.is_empty() {
        return if state.mode == Mode::Search {
            "No search matches.\nEsc: restore previous selection.".to_string()
        } else {
            "No panes discovered yet.".to_string()
        };
    }

    visible_targets
        .iter()
        .map(|target| {
            let selected = if state.selection.as_ref() == Some(target) {
                ">"
            } else {
                " "
            };
            let flash_label = if state.mode == Mode::FlashNavigate {
                state
                    .flash_label_for_target(target)
                    .map(|label| format!("[{label}] "))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            format!("{selected} {flash_label}{}", state.target_label(target))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn preview_lines(state: &AppState) -> String {
    let mut content = match state.selection.as_ref() {
        Some(SelectionTarget::Pane(pane_id)) => {
            if let Some(pane) = state.inventory.pane(pane_id) {
                let status = pane
                    .agent
                    .as_ref()
                    .map(|agent| status_label(agent.status))
                    .unwrap_or("NON-AGENT");
                let harness = pane
                    .agent
                    .as_ref()
                    .map(|agent| format!("{:?}", agent.harness))
                    .unwrap_or_else(|| "None".to_string());
                let preview = if pane.preview.trim().is_empty() {
                    "Preview capture is empty right now.".to_string()
                } else {
                    pane.preview.clone()
                };

                format!(
                    "Selected pane: {}\nStatus: {}\nHarness: {}\nCommand: {}\n\n{}",
                    pane.title,
                    status,
                    harness,
                    pane.current_command.as_deref().unwrap_or("unknown"),
                    preview
                )
            } else {
                "Selected pane is no longer available.".to_string()
            }
        }
        Some(SelectionTarget::Window(window_id)) => {
            if let Some(window) = state.inventory.window(window_id) {
                format!(
                    "Selected window: {}\nVisible panes: {}\n\nSelect a pane for detailed preview.",
                    window.name,
                    window.panes.len()
                )
            } else {
                "Selected window is no longer available.".to_string()
            }
        }
        Some(SelectionTarget::Session(session_id)) => {
            if let Some(session) = state.inventory.session(session_id) {
                format!(
                    "Selected session: {}\nWindows: {}\n\nExpand or choose a pane to inspect work.",
                    session.name,
                    session.windows.len()
                )
            } else {
                "Selected session is no longer available.".to_string()
            }
        }
        None => "Select a pane to inspect recent output and status.".to_string(),
    };

    if let Some(workspace_path) = state.selected_workspace_path() {
        content.push_str(&format!("\n\nWorkspace: {}", workspace_path.display()));
    }

    if let Some(lookup) = state.selected_pull_request_lookup() {
        match lookup {
            PullRequestLookup::Unknown => content.push_str("\nPR: checking"),
            PullRequestLookup::Missing => content.push_str("\nPR: no open pull request"),
            PullRequestLookup::Unavailable { .. } => content.push_str("\nPR: unavailable"),
            PullRequestLookup::Available(pull_request) => {
                content.push_str(&format!(
                    "\nPR: #{} {} - {}",
                    pull_request.number,
                    pull_request.status.label(),
                    pull_request.title
                ));
                if state.is_pull_request_detail_open() {
                    content.push_str(&format!(
                        "\n\nPull Request\n#{} {}\nTitle: {}\nRepo: {}\nBranches: {} -> {}\nAuthor: {}\nURL: {}\nActions: p toggle | O open | Y copy",
                        pull_request.number,
                        pull_request.status.label(),
                        pull_request.title,
                        pull_request.repository,
                        pull_request.branch,
                        pull_request.base_branch,
                        pull_request.author,
                        pull_request.url
                    ));
                }
            }
        }
    }

    content
}

fn status_label(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Working => "WORKING",
        AgentStatus::NeedsAttention => "ATTN",
        AgentStatus::Idle => "IDLE",
        AgentStatus::Error => "ERROR",
        AgentStatus::Unknown => "UNKNOWN",
    }
}

fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::app::{
        inventory, AgentStatus, AppState, FlashNavigateKind, FlashState, Focus, HarnessKind,
        ModalState, Mode, PaneBuilder, SearchState, SelectionTarget, SessionBuilder, WindowBuilder,
    };
    use crate::services::pull_requests::{PullRequestData, PullRequestLookup, PullRequestStatus};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::PathBuf;

    fn sample_state() -> AppState {
        let inventory = inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .title("claude-main")
                        .working_dir("/tmp/alpha")
                        .status(AgentStatus::Working)
                        .activity_score(10),
                ),
            ),
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:agents").pane(
                    PaneBuilder::agent("beta:codex", HarnessKind::CodexCli)
                        .title("codex-review")
                        .working_dir("/tmp/beta")
                        .status(AgentStatus::NeedsAttention)
                        .activity_score(4),
                ),
            ),
        ]);

        let mut state = AppState::with_inventory(inventory);
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));
        state
    }

    fn render_to_string(state: &AppState) -> String {
        let backend = TestBackend::new(100, 32);
        let mut terminal = Terminal::new(backend).expect("terminal should initialize");

        terminal
            .draw(|frame| render(frame, state))
            .expect("render should succeed");

        let buffer = terminal.backend().buffer();
        let area = buffer.area();
        let mut output = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                output.push_str(buffer[(x, y)].symbol());
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn render_includes_main_surfaces() {
        let state = sample_state();
        let output = render_to_string(&state);

        assert!(output.contains("Foreman"));
        assert!(output.contains("* Sidebar"));
        assert!(output.contains("Preview"));
        assert!(output.contains("Input"));
        assert!(output.contains("Footer"));
    }

    #[test]
    fn render_marks_focused_panel() {
        let mut state = sample_state();
        state.focus = Focus::Preview;
        let output = render_to_string(&state);

        assert!(output.contains("* Preview"));
        assert!(!output.contains("* Sidebar"));
    }

    #[test]
    fn render_marks_input_focus() {
        let mut state = sample_state();
        state.focus = Focus::Input;
        let output = render_to_string(&state);

        assert!(output.contains("* Input"));
        assert!(!output.contains("* Sidebar"));
    }

    #[test]
    fn render_shows_mode_in_footer() {
        let mut state = sample_state();
        state.mode = Mode::Search;
        let output = render_to_string(&state);

        assert!(output.contains("MODE: SEARCH"));
    }

    #[test]
    fn render_shows_startup_error_in_empty_shell() {
        let state = AppState {
            startup_error: Some("tmux unavailable".to_string()),
            ..AppState::default()
        };
        let output = render_to_string(&state);

        assert!(output.contains("tmux unavailable"));
        assert!(output.contains("No panes discovered yet.") || output.contains("Startup issue"));
    }

    #[test]
    fn render_displays_help_overlay() {
        let mut state = sample_state();
        state.mode = Mode::Help;
        let output = render_to_string(&state);

        assert!(output.contains("Help"));
        assert!(output.contains("j/k or arrows"));
    }

    #[test]
    fn render_shows_input_draft_and_submit_hint() {
        let mut state = sample_state();
        state.focus = Focus::Input;
        state.mode = Mode::Input;
        state.input_draft.text = "hello\nworld".to_string();
        let output = render_to_string(&state);

        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        assert!(output.contains("Ctrl+S: send"));
    }

    #[test]
    fn render_displays_confirm_kill_modal() {
        let mut state = sample_state();
        state.mode = Mode::ConfirmKill;
        state.modal = Some(ModalState::confirm_kill("alpha:claude".into()));
        let output = render_to_string(&state);

        assert!(output.contains("Confirm Kill"));
        assert!(output.contains("alpha:claude"));
        assert!(output.contains("Enter or y"));
    }

    #[test]
    fn render_displays_search_overlay_and_match_count() {
        let mut state = sample_state();
        state.mode = Mode::Search;
        let mut search = SearchState::new(state.selection.clone());
        search.draft.text = "codex".to_string();
        state.search = Some(search);
        state.selection = Some(SelectionTarget::Pane("beta:codex".into()));
        let output = render_to_string(&state);

        assert!(output.contains("Search"));
        assert!(output.contains("codex"));
        assert!(output.contains("Matches: 1"));
    }

    #[test]
    fn render_shows_fixed_width_flash_labels() {
        let inventory = inventory([SessionBuilder::new("alpha").window({
            let mut window = WindowBuilder::new("alpha:agents");
            for index in 0..27 {
                window = window.pane(
                    PaneBuilder::agent(format!("alpha:pane:{index}"), HarnessKind::ClaudeCode)
                        .title(format!("pane-{index}"))
                        .working_dir("/tmp/alpha")
                        .status(AgentStatus::Working),
                );
            }
            window
        })]);
        let mut state = AppState::with_inventory(inventory);
        state.mode = Mode::FlashNavigate;
        state.flash = Some(FlashState::new(
            state.selection.clone(),
            FlashNavigateKind::Jump,
        ));
        let output = render_to_string(&state);

        assert!(output.contains("Flash"));
        assert!(output.contains("[aa]"));
        assert!(output.contains("[ab]"));
    }

    #[test]
    fn render_shows_pull_request_compact_and_detail_sections() {
        let mut state = sample_state();
        state.pull_request_cache.insert(
            PathBuf::from("/tmp/alpha"),
            PullRequestLookup::Available(PullRequestData {
                number: 42,
                title: "Add PR awareness".to_string(),
                url: "https://example.com/pr/42".to_string(),
                repository: "foreman".to_string(),
                branch: "feat/pr-awareness".to_string(),
                base_branch: "main".to_string(),
                author: "alex".to_string(),
                status: PullRequestStatus::Open,
            }),
        );
        state.pull_request_detail_workspace = Some(PathBuf::from("/tmp/alpha"));

        let output = render_to_string(&state);

        assert!(output.contains("pr=#42 OPEN"));
        assert!(output.contains("Pull Request"));
        assert!(output.contains("feat/pr-awareness"));
        assert!(output.contains("https://example.com/pr/42"));
    }

    #[test]
    fn render_shows_missing_pull_request_without_detail_panel() {
        let mut state = sample_state();
        state
            .pull_request_cache
            .insert(PathBuf::from("/tmp/alpha"), PullRequestLookup::Missing);

        let output = render_to_string(&state);

        assert!(output.contains("pr=NONE"));
        assert!(output.contains("PR: no open pull request"));
        assert!(!output.contains("Pull Request"));
    }
}
