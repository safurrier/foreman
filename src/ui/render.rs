use crate::app::{AgentStatus, AppState, Focus, SelectionTarget};
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

    if state.mode == crate::app::Mode::Help {
        render_help(frame);
    }
}

fn render_header(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let visible_count = state.visible_targets().len();
    let content = format!(
        "Foreman | mode={} | focus={} | visible_targets={}",
        state.mode_label(),
        state.focus_label(),
        visible_count
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
    let hint = match state.focus {
        Focus::Input => "Input focused. Multiline compose will land here.",
        _ => "Direct input is available for the selected pane.",
    };
    let input = Paragraph::new(hint)
        .block(focused_block("Input", state.focus == Focus::Input))
        .wrap(Wrap { trim: false });
    frame.render_widget(input, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let footer = Paragraph::new(format!(
        "MODE: {} | FOCUS: {} | ? Help | q Quit",
        state.mode_label(),
        state.focus_label()
    ))
    .block(Block::default().borders(Borders::ALL).title("Footer"));
    frame.render_widget(footer, area);
}

fn render_help(frame: &mut Frame<'_>) {
    let popup = centered_rect(64, 45, frame.area());
    frame.render_widget(Clear, popup);
    let help = Paragraph::new(
        "Help\n\nj/k or arrows: move\nEnter: select\nTab or focus command: move focus\n?: toggle help\nq: quit",
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
        return "No panes discovered yet.".to_string();
    }

    visible_targets
        .iter()
        .map(|target| {
            let selected = if state.selection.as_ref() == Some(target) {
                ">"
            } else {
                " "
            };
            format!("{selected} {}", label_for_target(state, target))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn preview_lines(state: &AppState) -> String {
    match state.selection.as_ref() {
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
    }
}

fn label_for_target(state: &AppState, target: &SelectionTarget) -> String {
    match target {
        SelectionTarget::Session(session_id) => state
            .inventory
            .session(session_id)
            .map(|session| format!("Session  {}", session.name))
            .unwrap_or_else(|| format!("Session  {}", session_id.as_str())),
        SelectionTarget::Window(window_id) => state
            .inventory
            .window(window_id)
            .map(|window| format!("Window   {}", window.name))
            .unwrap_or_else(|| format!("Window   {}", window_id.as_str())),
        SelectionTarget::Pane(pane_id) => state
            .inventory
            .pane(pane_id)
            .map(|pane| {
                let status = pane
                    .agent
                    .as_ref()
                    .map(|agent| status_label(agent.status))
                    .unwrap_or("NON-AGENT");
                format!("Pane     {} [{}]", pane.title, status)
            })
            .unwrap_or_else(|| format!("Pane     {}", pane_id.as_str())),
    }
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
        inventory, AgentStatus, AppState, Focus, HarnessKind, Mode, PaneBuilder, SelectionTarget,
        SessionBuilder, WindowBuilder,
    };
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn sample_state() -> AppState {
        let inventory = inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .title("claude-main")
                        .status(AgentStatus::Working)
                        .activity_score(10),
                ),
            ),
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:agents").pane(
                    PaneBuilder::agent("beta:codex", HarnessKind::CodexCli)
                        .title("codex-review")
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
}
