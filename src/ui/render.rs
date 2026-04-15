use crate::app::{
    AgentStatus, AppState, Focus, HarnessKind, ModalState, Mode, Pane, SelectionTarget,
};
use crate::services::pull_requests::PullRequestLookup;
use crate::ui::theme::{Theme, ThemeName};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, state: &AppState, theme_name: ThemeName) {
    let theme = theme_name.resolve();
    let layout_mode = LayoutMode::for_area(frame.area());
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_header(frame, vertical[0], state, theme_name, &theme, layout_mode);
    render_body(frame, vertical[1], state, &theme, layout_mode);
    render_footer(frame, vertical[2], state, &theme, layout_mode);

    if state.mode == Mode::Help {
        render_help(frame, state, &theme, layout_mode);
    } else if state.modal.is_some() {
        render_modal(frame, state, &theme, layout_mode);
    } else if state.mode == Mode::Search {
        render_search_overlay(frame, state, &theme, layout_mode);
    } else if state.mode == Mode::FlashNavigate {
        render_flash_overlay(frame, state, &theme, layout_mode);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutMode {
    Compact,
    Medium,
    Wide,
}

impl LayoutMode {
    fn for_area(area: Rect) -> Self {
        if area.width < 96 || area.height < 28 {
            Self::Compact
        } else if area.width < 136 || area.height < 36 {
            Self::Medium
        } else {
            Self::Wide
        }
    }
}

fn render_header(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme_name: ThemeName,
    theme: &Theme,
    layout_mode: LayoutMode,
) {
    let visible_count = state.visible_targets().len();
    let pull_request = state
        .pull_request_compact_label()
        .unwrap_or_else(|| "pr=NONE".to_string());
    let alert = state.operator_alert_label().unwrap_or_default();
    let theme_label = format!("theme={}", theme_name.label());
    let alert_suffix = if alert.is_empty() {
        String::new()
    } else {
        format!(" | {alert}")
    };
    let content = match layout_mode {
        LayoutMode::Compact => format!(
            "Foreman | {} | {} | {} targets | {} | {}{}",
            state.mode_label(),
            state.system_stats_label(),
            visible_count,
            pull_request,
            state.notifications_label(),
            if alert.is_empty() {
                String::new()
            } else {
                format!(" | {alert}")
            }
        ),
        LayoutMode::Medium => format!(
            "Foreman | {} | {} | {} targets | {} | {} | {}",
            state.mode_label(),
            state.system_stats_label(),
            visible_count,
            state.filter_label(),
            pull_request,
            if alert.is_empty() {
                state.notifications_label()
            } else {
                alert.clone()
            }
        ),
        LayoutMode::Wide => format!(
            "Foreman | {} | {} | {} | {} targets | {} | {} | {} | {} | {}{}",
            state.mode_label(),
            state.focus_label(),
            state.system_stats_label(),
            visible_count,
            state.sort_label(),
            state.filter_label(),
            pull_request,
            state.notifications_label(),
            theme_label,
            alert_suffix
        ),
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled("Foreman", theme.emphasis),
        Span::styled(
            format!(" | {}", content.trim_start_matches("Foreman | ")),
            theme.base,
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Status")
            .border_style(theme.border),
    )
    .style(theme.base);
    frame.render_widget(header, area);
}

fn render_body(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    layout_mode: LayoutMode,
) {
    match layout_mode {
        LayoutMode::Compact => {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(42),
                    Constraint::Percentage(38),
                    Constraint::Percentage(20),
                ])
                .split(area);
            render_sidebar(frame, rows[0], state, theme);
            render_preview(frame, rows[1], state, theme, layout_mode);
            render_input(frame, rows[2], state, theme, layout_mode);
        }
        LayoutMode::Medium => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(area);
            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
                .split(columns[1]);

            render_sidebar(frame, columns[0], state, theme);
            render_preview(frame, right[0], state, theme, layout_mode);
            render_input(frame, right[1], state, theme, layout_mode);
        }
        LayoutMode::Wide => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(36), Constraint::Percentage(64)])
                .split(area);
            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(74), Constraint::Percentage(26)])
                .split(columns[1]);

            render_sidebar(frame, columns[0], state, theme);
            render_preview(frame, right[0], state, theme, layout_mode);
            render_input(frame, right[1], state, theme, layout_mode);
        }
    }
}

fn render_sidebar(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let title = if state.filters.harness.is_some() {
        format!("Targets [{}]", state.harness_filter_label())
    } else {
        "Targets".to_string()
    };
    let sidebar = Paragraph::new(sidebar_text(state, theme))
        .block(focused_block(title, state.focus == Focus::Sidebar, theme))
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(sidebar, area);
}

fn render_preview(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    layout_mode: LayoutMode,
) {
    let preview = Paragraph::new(preview_text(state, theme, layout_mode))
        .block(focused_block(
            "Details",
            state.focus == Focus::Preview,
            theme,
        ))
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(preview, area);
}

fn render_input(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    layout_mode: LayoutMode,
) {
    let input = Paragraph::new(input_text(state, theme, layout_mode))
        .block(focused_block("Compose", state.focus == Focus::Input, theme))
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(input, area);
}

fn render_footer(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    layout_mode: LayoutMode,
) {
    let footer = Paragraph::new(footer_text(state, theme, layout_mode))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(footer_title(state, theme))
                .border_style(theme.border),
        )
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(footer, area);
}

fn render_help(frame: &mut Frame<'_>, state: &AppState, theme: &Theme, layout_mode: LayoutMode) {
    let popup = help_popup_rect(frame.area(), layout_mode);
    frame.render_widget(Clear, popup);
    let help_lines = help_lines(state, theme);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(help_title(state, theme, help_lines.len() as u16, popup))
        .border_style(theme.overlay_border);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let (content_area, hint_area) = if inner.height >= 3 {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner);
        (rows[0], Some(rows[1]))
    } else {
        (inner, None)
    };

    let max_scroll = help_scroll_max(help_lines.len() as u16, content_area.height);
    let scroll = state.help_scroll.min(max_scroll);
    let help = Paragraph::new(Text::from(help_lines))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(help, content_area);

    if let Some(hint_area) = hint_area {
        let hint = Paragraph::new(Text::from(vec![help_hint_line(
            theme,
            layout_mode,
            scroll,
            max_scroll,
        )]))
        .style(theme.muted);
        frame.render_widget(hint, hint_area);
    }
}

fn render_search_overlay(
    frame: &mut Frame<'_>,
    state: &AppState,
    theme: &Theme,
    layout_mode: LayoutMode,
) {
    let popup = overlay_rect(frame.area(), layout_mode, 64, 9);
    frame.render_widget(Clear, popup);
    let query = state.search_query().unwrap_or("");
    let overlay = Paragraph::new(Text::from(vec![
        section_line("Search", theme),
        plain_line(""),
        plain_line(format!(
            "Query: {}",
            if query.is_empty() { "<empty>" } else { query }
        )),
        plain_line(format!("Matches: {}", state.visible_targets().len())),
        plain_line(""),
        muted_line(
            "Type to filter. Enter confirms. Esc restores the previous selection.",
            theme,
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search")
            .border_style(theme.search_border),
    )
    .wrap(Wrap { trim: false })
    .style(theme.base);
    frame.render_widget(overlay, popup);
}

fn render_flash_overlay(
    frame: &mut Frame<'_>,
    state: &AppState,
    theme: &Theme,
    layout_mode: LayoutMode,
) {
    let popup = overlay_rect(frame.area(), layout_mode, 64, 9);
    frame.render_widget(Clear, popup);
    let (mode_name, typed) = state
        .flash
        .as_ref()
        .map(|flash| {
            let mode_name = match flash.kind {
                crate::app::FlashNavigateKind::Jump => "jump",
                crate::app::FlashNavigateKind::JumpAndFocus => "jump+focus",
            };
            let typed = if flash.draft.text.is_empty() {
                "<empty>".to_string()
            } else {
                flash.draft.text.clone()
            };
            (mode_name, typed)
        })
        .unwrap_or(("jump", "<empty>".to_string()));
    let overlay = Paragraph::new(Text::from(vec![
        section_line("Flash", theme),
        plain_line(""),
        plain_line(format!("Mode: {mode_name}")),
        plain_line(format!("Typed: {typed}")),
        plain_line(format!("Labels: {}", state.flash_targets().len())),
        plain_line(""),
        muted_line("Type a visible label. Esc cancels.", theme),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Flash")
            .border_style(theme.warning_border),
    )
    .wrap(Wrap { trim: false })
    .style(theme.base);
    frame.render_widget(overlay, popup);
}

fn render_modal(frame: &mut Frame<'_>, state: &AppState, theme: &Theme, layout_mode: LayoutMode) {
    let Some(modal) = state.modal.as_ref() else {
        return;
    };

    let popup = modal_rect(frame.area(), layout_mode);
    frame.render_widget(Clear, popup);

    let (title, body, border_style) = match modal {
        ModalState::RenameWindow { window_id, draft } => (
            "Rename Window",
            Text::from(vec![
                section_line("Rename", theme),
                plain_line(""),
                plain_line(format!("Window: {}", window_id.as_str())),
                plain_line(""),
                plain_line(modal_draft_text(draft, "Type a new window name.")),
                plain_line(""),
                muted_line("Enter applies. Esc cancels.", theme),
            ]),
            theme.overlay_border,
        ),
        ModalState::SpawnWindow { session_id, draft } => (
            "Spawn Agent",
            Text::from(vec![
                section_line("Spawn", theme),
                plain_line(""),
                plain_line(format!("Session: {}", session_id.as_str())),
                plain_line(""),
                plain_line(modal_draft_text(
                    draft,
                    "Type the command for the new window.",
                )),
                plain_line(""),
                muted_line("Enter spawns. Esc cancels.", theme),
            ]),
            theme.overlay_border,
        ),
        ModalState::ConfirmKill { pane_id } => (
            "Confirm Kill",
            Text::from(vec![
                section_line("Kill Pane", theme),
                plain_line(""),
                plain_line(format!("Pane: {}", pane_id.as_str())),
                plain_line(""),
                muted_line("Enter or y confirms. Esc or n cancels.", theme),
            ]),
            theme.warning_border,
        ),
    };

    let modal = Paragraph::new(body)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false })
        .style(theme.base);
    frame.render_widget(modal, popup);
}

fn focused_block(title: impl Into<String>, focused: bool, theme: &Theme) -> Block<'static> {
    let title = title.into();
    let title = if focused { format!("* {title}") } else { title };

    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme.focus_border
        } else {
            theme.border
        })
}

fn sidebar_text(state: &AppState, theme: &Theme) -> Text<'static> {
    if let Some(error) = &state.startup_error {
        if state.inventory.sessions.is_empty() {
            return Text::from(vec![
                section_line("Startup issue", theme),
                plain_line(""),
                plain_line(error.clone()),
            ]);
        }
    }

    let visible_targets = state.visible_targets();
    if visible_targets.is_empty() {
        return if state.mode == Mode::Search {
            Text::from(vec![
                plain_line("No matches."),
                muted_line("Esc restores the previous selection.", theme),
            ])
        } else {
            Text::from(vec![plain_line("No panes discovered yet.")])
        };
    }

    Text::from(
        visible_targets
            .iter()
            .map(|target| sidebar_line(state, theme, target))
            .collect::<Vec<_>>(),
    )
}

fn sidebar_line(state: &AppState, theme: &Theme, target: &SelectionTarget) -> Line<'static> {
    let selected = state.selection.as_ref() == Some(target);
    let mut spans = Vec::new();

    spans.push(Span::styled(
        if selected {
            format!("{} ", theme.glyphs.selected)
        } else {
            "  ".to_string()
        },
        if selected {
            theme.selected
        } else {
            theme.muted
        },
    ));

    if state.mode == Mode::FlashNavigate {
        let flash_label = state
            .flash_label_for_target(target)
            .map(|label| format!("[{label}] "))
            .unwrap_or_default();
        if !flash_label.is_empty() {
            spans.push(Span::styled(flash_label, theme.warning_border));
        }
    }

    match target {
        SelectionTarget::Session(session_id) => {
            let (name, collapsed, rank, summary, marks) = state
                .inventory
                .session(session_id)
                .map(|session| {
                    let visible_windows = session.visible_windows(&state.filters, state.sort_mode);
                    let visible_panes = visible_windows
                        .iter()
                        .map(|window| window.visible_panes(&state.filters, state.sort_mode).len())
                        .sum::<usize>();
                    let marks = harness_marks_for_panes(
                        theme,
                        visible_windows.iter().flat_map(|window| {
                            window.visible_panes(&state.filters, state.sort_mode)
                        }),
                    );
                    (
                        session.name.clone(),
                        state.collapsed_sessions.contains(session_id),
                        session.attention_rank(),
                        format!("{}w/{}p", visible_windows.len(), visible_panes),
                        marks,
                    )
                })
                .unwrap_or_else(|| {
                    (
                        session_id.as_str().to_string(),
                        state.collapsed_sessions.contains(session_id),
                        AgentStatus::Unknown.attention_rank(),
                        "0w/0p".to_string(),
                        String::new(),
                    )
                });
            spans.push(Span::styled(
                format!(
                    "{} ",
                    if collapsed {
                        theme.glyphs.session_closed
                    } else {
                        theme.glyphs.session_open
                    }
                ),
                attention_style_from_rank(theme, rank),
            ));
            spans.push(Span::styled(
                name,
                if selected {
                    theme.selected
                } else {
                    theme.emphasis
                },
            ));
            spans.push(Span::styled(
                format!("  {summary}"),
                if selected {
                    theme.selected
                } else {
                    theme.muted
                },
            ));
            if !marks.is_empty() {
                spans.push(Span::styled(
                    format!("  {marks}"),
                    if selected {
                        theme.selected
                    } else {
                        theme.emphasis
                    },
                ));
            }
        }
        SelectionTarget::Window(window_id) => {
            let (name, rank, summary, marks) = state
                .inventory
                .window(window_id)
                .map(|window| {
                    let visible_panes = window.visible_panes(&state.filters, state.sort_mode);
                    let marks = harness_marks_for_panes(theme, visible_panes.iter().copied());
                    (
                        window.navigation_title(),
                        window.attention_rank(),
                        format!("{}p", visible_panes.len()),
                        marks,
                    )
                })
                .unwrap_or_else(|| {
                    (
                        window_id.as_str().to_string(),
                        AgentStatus::Unknown.attention_rank(),
                        "0p".to_string(),
                        String::new(),
                    )
                });
            spans.push(Span::styled("  ", theme.muted));
            spans.push(Span::styled("· ", attention_style_from_rank(theme, rank)));
            spans.push(Span::styled(
                name,
                if selected { theme.selected } else { theme.base },
            ));
            spans.push(Span::styled(
                format!(" {summary}"),
                if selected {
                    theme.selected
                } else {
                    theme.muted
                },
            ));
            if !marks.is_empty() {
                spans.push(Span::styled(
                    format!("  {marks}"),
                    if selected {
                        theme.selected
                    } else {
                        theme.emphasis
                    },
                ));
            }
        }
        SelectionTarget::Pane(pane_id) => {
            let pane = state.inventory.pane(pane_id);
            let status = pane.and_then(|pane| pane.agent.as_ref().map(|agent| agent.status));
            spans.push(Span::styled("    ", theme.muted));
            spans.push(Span::styled(
                format!(
                    "{} ",
                    status_symbol(theme, status, pane.is_some_and(|pane| pane.is_agent()))
                ),
                status_style(theme, status, pane.is_some_and(|pane| pane.is_agent())),
            ));
            spans.push(Span::styled(
                format!("{} ", pane_harness_badge(theme, pane)),
                if let Some(pane) = pane {
                    if pane.is_agent() {
                        status_style(theme, status, true)
                    } else {
                        theme.muted
                    }
                } else {
                    theme.muted
                },
            ));
            spans.push(Span::styled(
                pane.map(Pane::navigation_title)
                    .unwrap_or_else(|| pane_id.as_str().to_string()),
                if selected { theme.selected } else { theme.base },
            ));
        }
    }

    Line::from(spans)
}

fn preview_text(state: &AppState, theme: &Theme, layout_mode: LayoutMode) -> Text<'static> {
    if let Some(error) = &state.startup_error {
        if state.inventory.sessions.is_empty() {
            return Text::from(vec![
                section_line("tmux unavailable", theme),
                plain_line(""),
                plain_line(error.clone()),
            ]);
        }
    }

    let mut lines = Vec::new();

    if let Some(alert) = &state.operator_alert {
        let style = match alert.level {
            crate::app::OperatorAlertLevel::Info => theme.overlay_border,
            crate::app::OperatorAlertLevel::Warn => theme.warning_border,
            crate::app::OperatorAlertLevel::Error => theme.danger_border,
        };
        lines.push(Line::from(vec![
            Span::styled(format!("Alert [{}] ", alert.level.label()), style),
            Span::styled(alert.source.label(), theme.muted),
        ]));
        lines.push(plain_line(alert.message.clone()));
        lines.push(plain_line(""));
    }

    match state.selection.as_ref() {
        Some(SelectionTarget::Pane(pane_id)) => {
            if let Some(pane) = state.inventory.pane(pane_id) {
                let status = pane.agent.as_ref().map(|agent| agent.status);
                if let Some(breadcrumb) = state.selection_breadcrumb() {
                    lines.push(Line::from(vec![Span::styled(breadcrumb, theme.emphasis)]));
                }
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{} ", status_symbol(theme, status, pane.is_agent())),
                        status_style(theme, status, pane.is_agent()),
                    ),
                    Span::styled(
                        format!("{} ", pane_harness_badge(theme, Some(pane))),
                        theme.emphasis,
                    ),
                    Span::styled(pane.navigation_title(), theme.emphasis),
                ]));
                lines.push(plain_line(format!(
                    "Command: {}",
                    pane.current_command.as_deref().unwrap_or("unknown"),
                )));
                lines.push(status_source_line(theme, pane));
                lines.push(plain_line(format!("Pane title: {}", pane.title)));
                lines.push(muted_line(
                    "f jumps tmux here. i composes here. x kill.",
                    theme,
                ));
            } else {
                lines.push(plain_line("Selected pane is no longer available."));
            }
        }
        Some(SelectionTarget::Window(window_id)) => {
            if let Some(window) = state.inventory.window(window_id) {
                if let Some(breadcrumb) = state.selection_breadcrumb() {
                    lines.push(Line::from(vec![Span::styled(breadcrumb, theme.emphasis)]));
                }
                lines.push(Line::from(vec![Span::styled(
                    format!("Window {}", window.navigation_title()),
                    theme.emphasis,
                )]));
                lines.push(plain_line(format!(
                    "Visible panes: {}",
                    window.visible_panes(&state.filters, state.sort_mode).len()
                )));
                lines.extend(actionable_target_lines(state, theme));
                lines.push(muted_line(
                    "Enter or f jumps tmux to the target pane. i composes there.",
                    theme,
                ));
            } else {
                lines.push(plain_line("Selected window is no longer available."));
            }
        }
        Some(SelectionTarget::Session(session_id)) => {
            if let Some(session) = state.inventory.session(session_id) {
                if let Some(breadcrumb) = state.selection_breadcrumb() {
                    lines.push(Line::from(vec![Span::styled(breadcrumb, theme.emphasis)]));
                }
                lines.push(Line::from(vec![Span::styled(
                    format!("Session {}", session.name),
                    theme.emphasis,
                )]));
                let visible_windows = session.visible_windows(&state.filters, state.sort_mode);
                let visible_panes = visible_windows
                    .iter()
                    .map(|window| window.visible_panes(&state.filters, state.sort_mode).len())
                    .sum::<usize>();
                lines.push(plain_line(format!(
                    "Visible: {} windows / {} panes",
                    visible_windows.len(),
                    visible_panes
                )));
                lines.extend(actionable_target_lines(state, theme));
                lines.push(muted_line(
                    "Enter collapses. f jumps tmux to the target pane. i composes there.",
                    theme,
                ));
            } else {
                lines.push(plain_line("Selected session is no longer available."));
            }
        }
        None => {
            lines.push(plain_line(
                "Select a pane to inspect recent output and send work.",
            ));
        }
    }

    if let Some(workspace_path) = state.selected_workspace_path() {
        lines.push(plain_line(format!(
            "Workspace: {}",
            workspace_path.display()
        )));
    }

    let setup_lines = diagnostic_lines(state, theme);
    if !setup_lines.is_empty() {
        lines.push(plain_line(""));
        lines.extend(setup_lines);
    }

    lines.push(plain_line(format!(
        "View: {} {} {}",
        state.sort_label(),
        theme.glyphs.separator,
        state.filter_label()
    )));
    lines.push(plain_line(format!(
        "Notifications: {}",
        if state.notifications.muted {
            "muted".to_string()
        } else {
            state.notifications.profile.label().to_ascii_lowercase()
        }
    )));
    if let Some(status) = &state.notifications.last_status {
        lines.push(plain_line(format!("Notice: {status}")));
    }
    lines.push(plain_line(format!(
        "PR panel: {}",
        state.selected_pull_request_panel_label()
    )));

    lines.extend(pull_request_lines(state, theme));

    if let Some(SelectionTarget::Pane(pane_id)) = state.selection.as_ref() {
        if let Some(pane) = state.inventory.pane(pane_id) {
            lines.push(plain_line(""));
            lines.push(section_line("Recent output", theme));
            for line in preview_excerpt(&pane.preview, preview_line_limit(layout_mode)) {
                lines.push(plain_line(line));
            }
        }
    }

    Text::from(lines)
}

fn input_text(state: &AppState, theme: &Theme, layout_mode: LayoutMode) -> Text<'static> {
    let mut lines = Vec::new();
    let selected_pane = state.selected_actionable_pane();
    let target_label = selected_pane
        .map(|pane| {
            format!(
                "{} {}",
                pane_harness_badge(theme, Some(pane)),
                pane.navigation_title()
            )
        })
        .unwrap_or_else(|| "selected pane".to_string());

    if state.mode == Mode::Input {
        lines.push(Line::from(vec![Span::styled(
            format!("Compose for {target_label}"),
            theme.emphasis,
        )]));
        lines.push(muted_line(
            format!(
                "Enter sends {} Ctrl+J newline {} Esc cancels",
                theme.glyphs.separator, theme.glyphs.separator
            ),
            theme,
        ));
        lines.push(plain_line(""));
        if state.input_draft.text.is_empty() {
            lines.push(muted_line("Start typing your instruction.", theme));
        } else {
            for line in state.input_draft.text.lines() {
                lines.push(plain_line(line.to_string()));
            }
        }
        return Text::from(lines);
    }

    if !state.input_draft.text.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            format!("Draft for {target_label}"),
            theme.emphasis,
        )]));
        lines.push(muted_line(
            "Press i to resume or Esc to leave compose mode.",
            theme,
        ));
        lines.push(plain_line(""));
        for line in state.input_draft.text.lines().take(match layout_mode {
            LayoutMode::Compact => 1,
            LayoutMode::Medium => 2,
            LayoutMode::Wide => 3,
        }) {
            lines.push(plain_line(line.to_string()));
        }
        return Text::from(lines);
    }

    if selected_pane.is_some() {
        lines.push(Line::from(vec![Span::styled(
            format!("Compose -> {target_label}"),
            theme.emphasis,
        )]));
        if state.focus == Focus::Input {
            lines.push(muted_line(
                format!(
                    "Enter starts compose {} f jumps to tmux",
                    theme.glyphs.separator
                ),
                theme,
            ));
        } else {
            lines.push(muted_line(
                format!(
                    "i compose {} f jumps to tmux {} Tab or 3 focuses this panel",
                    theme.glyphs.separator, theme.glyphs.separator
                ),
                theme,
            ));
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "Select a row with an agent, then compose.",
            theme.emphasis,
        )]));
    }

    Text::from(lines)
}

fn footer_text(state: &AppState, theme: &Theme, layout_mode: LayoutMode) -> Text<'static> {
    let sep = format!(" {} ", theme.glyphs.separator);
    let lines = match state.mode {
        Mode::Input => vec![format!("Compose: Enter send{sep}Ctrl+J newline{sep}Esc cancel")],
        Mode::Search => vec![format!("Search: type filter{sep}Enter use match{sep}Esc restore")],
        Mode::FlashNavigate => vec![format!("Flash: type label{sep}Esc cancel")],
        Mode::Rename => vec![format!("Rename: Enter apply{sep}Esc cancel")],
        Mode::Spawn => vec![format!("Spawn: Enter create{sep}Esc cancel")],
        Mode::ConfirmKill => vec![format!("Kill: Enter or y confirm{sep}Esc or n cancel")],
        Mode::Help => match layout_mode {
            LayoutMode::Compact => vec![format!("Help: j/k scroll{sep}PgUp or PgDn{sep}Esc close")],
            LayoutMode::Medium | LayoutMode::Wide => vec![format!(
                "Help: j/k or arrows scroll{sep}PgUp/PgDn page{sep}Home/End jump{sep}Esc close{sep}q quit"
            )],
        },
        Mode::Normal | Mode::PreviewScroll => normal_footer_lines(state, theme, layout_mode),
    };

    Text::from(lines.into_iter().map(plain_line).collect::<Vec<_>>())
}

fn help_lines(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = vec![
        section_line("Right now", theme),
        muted_line(format!("Focus: {}", state.focus_context_label()), theme),
        muted_line(state.focus_help_summary(), theme),
    ];

    if let Some(source_summary) = state.selected_actionable_source_summary() {
        lines.push(muted_line(
            format!("Target source: {source_summary}."),
            theme,
        ));
    }

    lines.extend([
        plain_line(""),
        section_line("Source", theme),
        muted_line(
            "native hook = higher-confidence status from harness signals.",
            theme,
        ),
        muted_line(
            "compatibility heuristic = tmux-observed status that may lag or be less certain.",
            theme,
        ),
        plain_line(""),
        section_line("Navigate", theme),
        muted_line("j/k or arrows move.", theme),
        muted_line("Enter uses the row. f jumps tmux to Target pane.", theme),
        muted_line("Tab or 1/2/3 changes panel focus.", theme),
        plain_line(""),
        section_line("Legend", theme),
        muted_line(
            format!(
                "Status: {} working  {} attention  {} idle  {} error  {} unknown",
                theme.glyphs.working,
                theme.glyphs.attention,
                theme.glyphs.idle,
                theme.glyphs.error,
                theme.glyphs.unknown
            ),
            theme,
        ),
        muted_line(
            format!(
                "Harness: {} Claude  {} Codex  {} Pi",
                theme.glyphs.claude, theme.glyphs.codex, theme.glyphs.pi
            ),
            theme,
        ),
        muted_line(
            format!(
                "Compat:  {} Gemini  {} OpenCode  {} shell",
                theme.glyphs.gemini, theme.glyphs.opencode, theme.glyphs.shell
            ),
            theme,
        ),
        plain_line(""),
        section_line("Target Pane", theme),
        muted_line("Target pane is what Enter, f, i, and x use.", theme),
        muted_line(
            "Session/window rows resolve to the best visible pane.",
            theme,
        ),
        plain_line(""),
        section_line("Act", theme),
        muted_line("i starts compose. Enter sends. Ctrl+J adds newline.", theme),
        muted_line("R renames the window. N spawns a new agent window.", theme),
        muted_line("Popup mode closes after successful tmux focus.", theme),
        plain_line(""),
        section_line("Find", theme),
        muted_line("/ filters the list. s jumps. S jumps and focuses.", theme),
        muted_line(
            "p opens PR detail. O opens in browser. Y copies URL.",
            theme,
        ),
        plain_line(""),
        section_line("View", theme),
        muted_line(
            "h cycles visible harnesses. H shows sessions. P shows panes.",
            theme,
        ),
        muted_line(
            "o sorts. t themes. m mutes. n changes notification profile.",
            theme,
        ),
    ]);

    lines
}

fn help_title(state: &AppState, theme: &Theme, total_lines: u16, popup: Rect) -> String {
    let inner_height = popup.height.saturating_sub(2);
    let content_height = if inner_height >= 3 {
        inner_height.saturating_sub(1)
    } else {
        inner_height
    }
    .max(1);
    let max_scroll = help_scroll_max(total_lines, content_height);
    let scroll = state.help_scroll.min(max_scroll);
    let page_size = content_height.max(1);
    let total_pages = total_lines.max(1).div_ceil(page_size);
    let current_page = (scroll / page_size).min(total_pages.saturating_sub(1)) + 1;
    format!(
        "Help {} {} {current_page}/{total_pages}",
        theme.glyphs.separator,
        state.focus_context_label()
    )
}

fn help_scroll_max(total_lines: u16, viewport_height: u16) -> u16 {
    total_lines.saturating_sub(viewport_height)
}

fn help_hint_line(
    theme: &Theme,
    layout_mode: LayoutMode,
    scroll: u16,
    max_scroll: u16,
) -> Line<'static> {
    let progress = if max_scroll == 0 {
        "all visible".to_string()
    } else {
        format!("line {} / {}", scroll + 1, max_scroll + 1)
    };
    let hint = match layout_mode {
        LayoutMode::Compact => format!(
            "Scroll j/k {} PgUp/PgDn {} Esc close {} {progress}",
            theme.glyphs.separator, theme.glyphs.separator, theme.glyphs.separator
        ),
        LayoutMode::Medium | LayoutMode::Wide => format!(
            "Scroll j/k or arrows {} PgUp/PgDn page {} Home/End jump {} Esc close {} {progress}",
            theme.glyphs.separator,
            theme.glyphs.separator,
            theme.glyphs.separator,
            theme.glyphs.separator
        ),
    };
    muted_line(hint, theme)
}

fn pull_request_lines(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    match state.selected_pull_request_lookup() {
        Some(PullRequestLookup::Unknown) => lines.push(plain_line("PR: checking")),
        Some(PullRequestLookup::Missing) => lines.push(plain_line("PR: no open pull request")),
        Some(PullRequestLookup::Unavailable { .. }) => lines.push(plain_line("PR: unavailable")),
        Some(PullRequestLookup::Available(pull_request)) => {
            lines.push(Line::from(vec![
                Span::styled(
                    format!(
                        "PR #{} {}",
                        pull_request.number,
                        pull_request.status.label()
                    ),
                    theme.emphasis,
                ),
                Span::styled(format!("  {}", pull_request.title), theme.base),
            ]));

            if state.is_pull_request_detail_open() {
                lines.push(plain_line(format!("Repo: {}", pull_request.repository)));
                lines.push(plain_line(format!(
                    "Branches: {} -> {}",
                    pull_request.branch, pull_request.base_branch
                )));
                lines.push(plain_line(format!("Author: {}", pull_request.author)));
                lines.push(muted_line(
                    "p toggles detail. O opens in the browser. Y copies the URL.",
                    theme,
                ));
            }
        }
        None => {}
    }

    lines
}

fn diagnostic_lines(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let diagnostics = state
        .selected_runtime_diagnostics()
        .into_iter()
        .filter(|finding| finding.severity != crate::doctor::DoctorSeverity::Ok)
        .take(2)
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        return Vec::new();
    }

    let mut lines = vec![section_line("Setup", theme)];
    for finding in diagnostics {
        let severity_style = match finding.severity {
            crate::doctor::DoctorSeverity::Error => theme.error,
            crate::doctor::DoctorSeverity::Warn => theme.attention,
            crate::doctor::DoctorSeverity::Info => theme.muted,
            crate::doctor::DoctorSeverity::Ok => theme.base,
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", finding.severity.label()), severity_style),
            Span::styled(finding.summary.clone(), theme.base),
        ]));
        if let Some(next_step) = &finding.next_step {
            lines.push(muted_line(next_step.clone(), theme));
        }
    }

    lines
}

fn preview_excerpt(preview: &str, max_lines: usize) -> Vec<String> {
    let mut lines = preview
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if lines.is_empty() {
        return vec!["Preview capture is empty right now.".to_string()];
    }

    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }

    lines
}

fn preview_line_limit(layout_mode: LayoutMode) -> usize {
    match layout_mode {
        LayoutMode::Compact => 4,
        LayoutMode::Medium => 7,
        LayoutMode::Wide => 11,
    }
}

fn modal_draft_text(draft: &crate::app::TextDraft, placeholder: &str) -> String {
    if draft.text.is_empty() {
        placeholder.to_string()
    } else {
        draft.text.clone()
    }
}

fn help_popup_rect(area: Rect, layout_mode: LayoutMode) -> Rect {
    match layout_mode {
        LayoutMode::Compact => inset_rect(area, 1, 1),
        LayoutMode::Medium => centered_rect(area, 78, 20),
        LayoutMode::Wide => centered_rect(area, 86, 22),
    }
}

fn modal_rect(area: Rect, layout_mode: LayoutMode) -> Rect {
    match layout_mode {
        LayoutMode::Compact => inset_rect(area, 2, 3),
        LayoutMode::Medium => centered_rect(area, 72, 12),
        LayoutMode::Wide => centered_rect(area, 78, 12),
    }
}

fn overlay_rect(area: Rect, layout_mode: LayoutMode, width: u16, height: u16) -> Rect {
    match layout_mode {
        LayoutMode::Compact => centered_rect(
            area,
            width.min(area.width.saturating_sub(4)),
            height.min(area.height.saturating_sub(4)),
        ),
        LayoutMode::Medium | LayoutMode::Wide => centered_rect(area, width, height),
    }
}

fn inset_rect(area: Rect, horizontal_margin: u16, vertical_margin: u16) -> Rect {
    let width = area
        .width
        .saturating_sub(horizontal_margin.saturating_mul(2));
    let height = area
        .height
        .saturating_sub(vertical_margin.saturating_mul(2));
    Rect {
        x: area.x.saturating_add(horizontal_margin),
        y: area.y.saturating_add(vertical_margin),
        width: width.max(1),
        height: height.max(1),
    }
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(2)).max(1);
    let height = height.min(area.height.saturating_sub(2)).max(1);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn pane_harness_badge(theme: &Theme, pane: Option<&Pane>) -> &'static str {
    harness_badge(theme, pane.and_then(Pane::harness_kind))
}

fn harness_badge(theme: &Theme, harness: Option<HarnessKind>) -> &'static str {
    match harness {
        Some(HarnessKind::ClaudeCode) => theme.glyphs.claude,
        Some(HarnessKind::CodexCli) => theme.glyphs.codex,
        Some(HarnessKind::Pi) => theme.glyphs.pi,
        Some(HarnessKind::GeminiCli) => theme.glyphs.gemini,
        Some(HarnessKind::OpenCode) => theme.glyphs.opencode,
        None => theme.glyphs.shell,
    }
}

fn harness_marks_for_panes<'a>(theme: &Theme, panes: impl IntoIterator<Item = &'a Pane>) -> String {
    let mut marks = Vec::new();
    let mut saw_shell = false;

    for pane in panes {
        match pane.harness_kind() {
            Some(harness) => {
                let badge = harness_badge(theme, Some(harness));
                if !marks.contains(&badge) {
                    marks.push(badge);
                }
            }
            None => saw_shell = true,
        }
    }

    if saw_shell && marks.is_empty() {
        marks.push(theme.glyphs.shell);
    }

    marks.concat()
}

fn actionable_target_lines(state: &AppState, theme: &Theme) -> Vec<Line<'static>> {
    let Some(pane) = state.selected_actionable_pane() else {
        return Vec::new();
    };
    let status = pane.agent.as_ref().map(|agent| agent.status);

    vec![
        Line::from(vec![
            Span::styled("Target pane: ", theme.muted),
            Span::styled(
                format!("{} ", status_symbol(theme, status, pane.is_agent())),
                status_style(theme, status, pane.is_agent()),
            ),
            Span::styled(
                format!("{} ", pane_harness_badge(theme, Some(pane))),
                status_style(theme, status, pane.is_agent()),
            ),
            Span::styled(pane.navigation_title(), theme.base),
        ]),
        status_source_line(theme, pane),
    ]
}

fn status_source_line(theme: &Theme, pane: &Pane) -> Line<'static> {
    match pane.agent.as_ref() {
        Some(agent) => Line::from(vec![
            Span::styled("Status source: ", theme.muted),
            Span::styled(agent.integration_mode.source_label(), theme.base),
            Span::styled(
                format!(
                    " {} {}",
                    theme.glyphs.separator,
                    agent.integration_mode.confidence_label()
                ),
                theme.muted,
            ),
        ]),
        None => muted_line("Status source: plain shell pane", theme),
    }
}

fn footer_title(state: &AppState, theme: &Theme) -> String {
    match state.mode {
        Mode::Normal | Mode::PreviewScroll => {
            format!(
                "Keys {} {}",
                theme.glyphs.separator,
                state.focus_context_label()
            )
        }
        Mode::Input => format!("Keys {} Compose", theme.glyphs.separator),
        Mode::Spawn => format!("Keys {} Spawn", theme.glyphs.separator),
        Mode::Search => format!("Keys {} Search", theme.glyphs.separator),
        Mode::FlashNavigate => format!("Keys {} Flash", theme.glyphs.separator),
        Mode::Rename => format!("Keys {} Rename", theme.glyphs.separator),
        Mode::Help => format!("Keys {} Help", theme.glyphs.separator),
        Mode::ConfirmKill => format!("Keys {} Confirm", theme.glyphs.separator),
    }
}

fn normal_footer_lines(state: &AppState, theme: &Theme, layout_mode: LayoutMode) -> Vec<String> {
    let sep = format!(" {} ", theme.glyphs.separator);
    let secondary = match layout_mode {
        LayoutMode::Compact => format!("View h/o/t{sep}Help ?"),
        LayoutMode::Medium => {
            format!("Panels Tab or 1/2/3{sep}Find / s S{sep}View h o t{sep}Help ?")
        }
        LayoutMode::Wide => {
            format!("Panels Tab or 1/2/3{sep}Find / s S{sep}View h o t{sep}Alerts m n{sep}Help ?")
        }
    };

    match state.focus {
        Focus::Sidebar => {
            let primary = if state.selected_actionable_pane().is_some() {
                format!("Sidebar: j/k move{sep}Enter use row{sep}f jump tmux{sep}i compose")
            } else {
                format!("Sidebar: j/k move{sep}Enter fold row{sep}Tab details{sep}Help ?")
            };
            match layout_mode {
                LayoutMode::Compact => vec![primary],
                LayoutMode::Medium | LayoutMode::Wide => vec![primary, secondary],
            }
        }
        Focus::Preview => {
            let primary = if state.selected_actionable_pane().is_some() {
                format!("Details: inspect target pane{sep}f jump tmux{sep}i compose")
            } else {
                format!("Details: inspect selection{sep}Tab sidebar{sep}Help ?")
            };
            let pr_actions = if state.selected_pull_request().is_some() {
                format!("PR p{sep}Open O{sep}Copy Y")
            } else {
                "PR p".to_string()
            };
            match layout_mode {
                LayoutMode::Compact => {
                    vec![format!(
                        "Details: inspect target{sep}f tmux{sep}p PR{sep}Help ?"
                    )]
                }
                LayoutMode::Medium | LayoutMode::Wide => {
                    vec![primary, format!("{secondary}{sep}{pr_actions}")]
                }
            }
        }
        Focus::Input => {
            let primary = if state.selected_actionable_pane().is_some() {
                format!("Compose: Enter or i start{sep}f jump tmux{sep}x kill")
            } else {
                format!("Compose: select an agent row first{sep}Tab sidebar{sep}Help ?")
            };
            match layout_mode {
                LayoutMode::Compact => vec![primary],
                LayoutMode::Medium | LayoutMode::Wide => vec![primary, secondary],
            }
        }
    }
}

fn status_symbol(theme: &Theme, status: Option<AgentStatus>, is_agent: bool) -> &'static str {
    if !is_agent {
        return theme.glyphs.non_agent;
    }

    match status.unwrap_or(AgentStatus::Unknown) {
        AgentStatus::Working => theme.glyphs.working,
        AgentStatus::NeedsAttention => theme.glyphs.attention,
        AgentStatus::Idle => theme.glyphs.idle,
        AgentStatus::Error => theme.glyphs.error,
        AgentStatus::Unknown => theme.glyphs.unknown,
    }
}

fn status_style(theme: &Theme, status: Option<AgentStatus>, is_agent: bool) -> Style {
    if !is_agent {
        return theme.non_agent;
    }

    match status.unwrap_or(AgentStatus::Unknown) {
        AgentStatus::Working => theme.working,
        AgentStatus::NeedsAttention => theme.attention,
        AgentStatus::Idle => theme.idle,
        AgentStatus::Error => theme.error,
        AgentStatus::Unknown => theme.unknown,
    }
}

fn attention_style_from_rank(theme: &Theme, rank: u8) -> Style {
    match rank {
        0 => theme.error,
        1 => theme.attention,
        2 => theme.working,
        3 => theme.idle,
        _ => theme.unknown,
    }
}

fn section_line(text: impl Into<String>, theme: &Theme) -> Line<'static> {
    Line::from(vec![Span::styled(text.into(), theme.emphasis)])
}

fn muted_line(text: impl Into<String>, theme: &Theme) -> Line<'static> {
    Line::from(vec![Span::styled(text.into(), theme.muted)])
}

fn plain_line(text: impl Into<String>) -> Line<'static> {
    Line::from(vec![Span::raw(text.into())])
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::app::{
        inventory, AgentStatus, AppState, FlashNavigateKind, FlashState, Focus, HarnessKind,
        IntegrationMode, ModalState, Mode, OperatorAlert, OperatorAlertLevel, OperatorAlertSource,
        PaneBuilder, SearchState, SelectionTarget, SessionBuilder, WindowBuilder,
    };
    use crate::doctor::{DoctorArea, DoctorFinding, DoctorSeverity};
    use crate::services::pull_requests::{PullRequestData, PullRequestLookup, PullRequestStatus};
    use crate::services::system_stats::SystemStatsSnapshot;
    use crate::ui::theme::ThemeName;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::PathBuf;

    fn sample_state() -> AppState {
        let inventory = inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").name("agents").pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .title("M1-AFurrier")
                        .current_command("claude")
                        .working_dir("/tmp/alpha")
                        .preview("Claude is working\nReading files\nApplying patch")
                        .status(AgentStatus::Working)
                        .integration_mode(IntegrationMode::Native)
                        .activity_score(10),
                ),
            ),
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:agents").name("review").pane(
                    PaneBuilder::agent("beta:codex", HarnessKind::CodexCli)
                        .title("M1-AFurrier")
                        .current_command("codex")
                        .working_dir("/tmp/foreman")
                        .preview("Codex waiting for your input\nsh-3.2$")
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
        render_to_string_at(state, ThemeName::Catppuccin, 100, 32)
    }

    fn render_to_string_at(
        state: &AppState,
        theme_name: ThemeName,
        width: u16,
        height: u16,
    ) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal should initialize");

        terminal
            .draw(|frame| render(frame, state, theme_name))
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
        assert!(output.contains("* Targets"));
        assert!(output.contains("Details"));
        assert!(output.contains("Compose"));
        assert!(output.contains("Keys"));
    }

    #[test]
    fn render_marks_focused_panel() {
        let mut state = sample_state();
        state.focus = Focus::Preview;
        let output = render_to_string(&state);

        assert!(output.contains("* Details"));
        assert!(!output.contains("* Targets"));
    }

    #[test]
    fn render_marks_input_focus() {
        let mut state = sample_state();
        state.focus = Focus::Input;
        let output = render_to_string(&state);

        assert!(output.contains("* Compose"));
        assert!(!output.contains("* Targets"));
    }

    #[test]
    fn render_shows_mode_in_header() {
        let mut state = sample_state();
        state.mode = Mode::Search;
        let output = render_to_string(&state);

        assert!(output.contains("SEARCH"));
    }

    #[test]
    fn render_shows_startup_error_in_empty_shell() {
        let state = AppState {
            startup_error: Some("tmux unavailable".to_string()),
            ..AppState::default()
        };
        let output = render_to_string(&state);

        assert!(output.contains("tmux unavailable"));
    }

    #[test]
    fn render_displays_help_overlay_with_extended_command_surface() {
        let mut state = sample_state();
        state.mode = Mode::Help;
        let output = render_to_string(&state);

        assert!(output.contains("Right now"));
        assert!(output.contains("Focus: Sidebar"));
        assert!(output.contains("Source"));
        assert!(output.contains("Navigate"));
        assert!(output.contains("Legend"));
        assert!(output.contains("native hook = higher-confidence"));
        assert!(output.contains("Scroll j/k or arrows"));
    }

    #[test]
    fn render_help_supports_scroll_offset_for_lower_sections() {
        let mut state = sample_state();
        state.mode = Mode::Help;
        let top_output = render_to_string_at(&state, ThemeName::Catppuccin, 80, 18);
        assert!(!top_output.contains("h cycles visible harnesses"));

        state.help_scroll = 24;
        let scrolled_output = render_to_string_at(&state, ThemeName::Catppuccin, 80, 18);
        assert!(scrolled_output.contains("h cycles visible harnesses"));
    }

    #[test]
    fn render_shows_input_draft_and_submit_hint() {
        let mut state = sample_state();
        state.focus = Focus::Input;
        state.mode = Mode::Input;
        state.input_draft.text = "hello\nworld".to_string();
        let output = render_to_string(&state);

        assert!(output.contains("Compose for"));
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
        assert!(output.contains("Enter sends"));
        assert!(output.contains("Ctrl+J newline"));
    }

    #[test]
    fn render_preview_names_resolved_target_pane() {
        let mut state = sample_state();
        state.selection = Some(SelectionTarget::Session("alpha".into()));
        let output = render_to_string(&state);

        assert!(output.contains("Target pane:"));
        assert!(output.contains("Status source: native hook"));
        assert!(output.contains("f jumps tmux to the target pane"));
    }

    #[test]
    fn render_preview_surfaces_status_provenance_for_selected_pane() {
        let state = sample_state();
        let output = render_to_string(&state);

        assert!(output.contains("Status source: native hook"));
        assert!(output.contains("high confidence"));
    }

    #[test]
    fn render_surfaces_runtime_setup_diagnostics_for_selected_pane() {
        let mut state = sample_state();
        state.runtime_diagnostics = vec![DoctorFinding::new(
            "claude-no-native-signals",
            DoctorSeverity::Warn,
            DoctorArea::Runtime,
            "No native signals were observed for 1 visible Claude Code pane(s).",
        )
        .with_provider(HarnessKind::ClaudeCode)
        .with_next_step("Run foreman --setup --repo /tmp/alpha")];

        let output = render_to_string(&state);

        assert!(output.contains("Setup"));
        assert!(output.contains("No native signals were observed"));
        assert!(output.contains("Run foreman --setup"));
    }

    #[test]
    fn render_displays_confirm_kill_modal() {
        let mut state = sample_state();
        state.mode = Mode::ConfirmKill;
        state.modal = Some(ModalState::confirm_kill("alpha:claude".into()));
        let output = render_to_string(&state);

        assert!(output.contains("Confirm Kill"));
        assert!(output.contains("alpha:claude"));
        assert!(output.contains("Enter or y confirms"));
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
            let mut window = WindowBuilder::new("alpha:agents").name("agents");
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
    fn render_sidebar_uses_harness_badges_and_workspace_titles() {
        let state = sample_state();
        let output = render_to_string(&state);

        assert!(output.contains("✦ alpha"));
        assert!(output.contains("◎ foreman"));
        assert!(!output.contains("Pane     "));
    }

    #[test]
    fn render_footer_uses_labeled_control_groups() {
        let state = sample_state();
        let output = render_to_string(&state);

        assert!(output.contains("Keys • Sidebar") || output.contains("Keys | Sidebar"));
        assert!(output.contains("Sidebar: j/k move"));
        assert!(output.contains("Enter use row"));
        assert!(output.contains("Find / s S"));
        assert!(output.contains("Help ?"));
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
        assert!(output.contains("PR panel: open"));
        assert!(output.contains("Repo: foreman"));
        assert!(output.contains("feat/pr-awareness"));
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
        assert!(output.contains("PR panel: none"));
    }

    #[test]
    fn render_reflects_notification_mute_and_profile_state() {
        let mut state = sample_state();
        state.notifications.muted = true;
        state.notifications.last_status = Some("Notifications muted".to_string());
        let muted_output = render_to_string(&state);
        assert!(muted_output.contains("notify=MUTED"));
        assert!(muted_output.contains("Notice: Notifications muted"));

        state.notifications.muted = false;
        state.notifications.profile = crate::app::NotificationProfile::CompletionOnly;
        state.notifications.last_status = Some("Notification profile: COMPLETE".to_string());
        let profile_output = render_to_string(&state);
        assert!(profile_output.contains("notify=COMPLETE"));
        assert!(profile_output.contains("Notifications: complete"));
    }

    #[test]
    fn render_includes_system_stats_in_header() {
        let mut state = sample_state();
        state.system_stats = SystemStatsSnapshot {
            cpu_pressure_percent: Some(18),
            memory_pressure_percent: Some(71),
        };

        let output = render_to_string(&state);

        assert!(output.contains("Foreman"));
        assert!(output.contains("18"));
    }

    #[test]
    fn render_surfaces_operator_alert_in_preview() {
        let mut state = sample_state();
        state.operator_alert = Some(OperatorAlert::new(
            OperatorAlertSource::PullRequests,
            OperatorAlertLevel::Warn,
            "PR lookup unavailable: GitHub CLI is not installed",
        ));

        let output = render_to_string(&state);

        assert!(output.contains("alert=WARN"));
        assert!(output.contains("Alert [WARN]"));
        assert!(output.contains("GitHub CLI is not installed"));
    }

    #[test]
    fn render_uses_ascii_fallbacks_for_no_color_theme() {
        let mut state = sample_state();
        state.selection = Some(SelectionTarget::Session("alpha".into()));
        let output = render_to_string_at(&state, ThemeName::NoColor, 80, 24);

        assert!(output.contains("> v alpha") || output.contains("> alpha"));
        assert!(output.contains("|") || output.contains("compose"));
    }
}
