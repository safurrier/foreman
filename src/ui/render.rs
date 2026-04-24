use crate::app::{
    AgentStatus, AppState, Focus, HarnessKind, ModalState, Mode, Pane, SelectionTarget,
    SidebarHarnessSummary, SidebarRowKind, VisibleTargetEntry,
};
use crate::services::pull_requests::PullRequestLookup;
use crate::ui::theme::{Theme, ThemeName};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
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

pub fn sidebar_viewport_rows_for_area(area: Rect) -> u16 {
    let layout_mode = LayoutMode::for_area(area);
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(area);
    let body = vertical[1];
    let sidebar = match layout_mode {
        LayoutMode::Compact => Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(42),
                Constraint::Percentage(38),
                Constraint::Percentage(20),
            ])
            .split(body)[0],
        LayoutMode::Medium => Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(body)[0],
        LayoutMode::Wide => Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(36), Constraint::Percentage(64)])
            .split(body)[0],
    };

    sidebar.height.saturating_sub(2)
}

fn render_header(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme_name: ThemeName,
    theme: &Theme,
    layout_mode: LayoutMode,
) {
    let visible_count = state.visible_target_count();
    let pull_request = state
        .pull_request_compact_label()
        .unwrap_or_else(|| "pr=NONE".to_string());
    let alert = state.operator_alert_label().unwrap_or_default();
    let startup_cache = state
        .startup_cache_age_ms
        .map(|age_ms| format!("cached {}", humanize_age_ms(age_ms)));
    let theme_label = format!("theme={}", theme_name.label());
    let alert_suffix = if alert.is_empty() {
        String::new()
    } else {
        format!(" | {alert}")
    };
    let content = match layout_mode {
        LayoutMode::Compact => format!(
            "Foreman | {} | {} | {} targets{} | {} | {}{}",
            state.mode_label(),
            state.system_stats_label(),
            visible_count,
            startup_cache
                .as_ref()
                .map(|label| format!(" | {label}"))
                .unwrap_or_default(),
            pull_request,
            state.notifications_label(),
            if alert.is_empty() {
                String::new()
            } else {
                format!(" | {alert}")
            }
        ),
        LayoutMode::Medium => format!(
            "Foreman | {} | {} | {} targets{} | {} | {} | {}",
            state.mode_label(),
            state.system_stats_label(),
            visible_count,
            startup_cache
                .as_ref()
                .map(|label| format!(" | {label}"))
                .unwrap_or_default(),
            state.filter_label(),
            pull_request,
            if alert.is_empty() {
                state.notifications_label()
            } else {
                alert.clone()
            }
        ),
        LayoutMode::Wide => format!(
            "Foreman | {} | {} | {} | {} targets{} | {} | {} | {} | {} | {}{}",
            state.mode_label(),
            state.focus_label(),
            state.system_stats_label(),
            visible_count,
            startup_cache
                .as_ref()
                .map(|label| format!(" | {label}"))
                .unwrap_or_default(),
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
    let block = focused_block(title, state.focus == Focus::Sidebar, theme);

    if let Some(placeholder) = sidebar_placeholder_text(state, theme) {
        let sidebar = Paragraph::new(placeholder)
            .block(block)
            .wrap(Wrap { trim: false })
            .style(theme.base);
        frame.render_widget(sidebar, area);
        return;
    }

    let items = state
        .visible_target_entries()
        .iter()
        .map(|entry| ListItem::new(sidebar_line(state, theme, entry)))
        .collect::<Vec<_>>();
    let mut list_state = ListState::default()
        .with_offset(state.sidebar_scroll())
        .with_selected(state.selected_visible_index());
    let list = List::new(items).block(block).style(theme.base);
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_preview(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    layout_mode: LayoutMode,
) {
    let text = preview_text(state, theme, layout_mode);
    let content_height = area.height.saturating_sub(2).max(1);
    let max_scroll = text
        .lines
        .len()
        .saturating_sub(content_height as usize)
        .min(u16::MAX as usize) as u16;
    let scroll = state.preview_scroll.min(max_scroll);
    let preview = Paragraph::new(text)
        .block(focused_block(
            "Details",
            state.focus == Focus::Preview,
            theme,
        ))
        .scroll((scroll, 0))
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
        plain_line(format!("Matches: {}", state.visible_target_count())),
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

fn sidebar_placeholder_text(state: &AppState, theme: &Theme) -> Option<Text<'static>> {
    if let Some(error) = &state.startup_error {
        if state.inventory.sessions.is_empty() {
            return Some(Text::from(vec![
                section_line("Startup issue", theme),
                plain_line(""),
                plain_line(error.clone()),
            ]));
        }
    }

    let visible_entries = state.visible_target_entries();
    if visible_entries.is_empty() {
        return Some(if state.mode == Mode::Search {
            Text::from(vec![
                plain_line("No matches."),
                muted_line("Esc restores the previous selection.", theme),
            ])
        } else if state.startup_loading {
            Text::from(vec![
                section_line("Loading tmux inventory", theme),
                plain_line(""),
                muted_line(
                    "Foreman is drawing immediately and backfilling sessions, panes, and previews in the background.",
                    theme,
                ),
            ])
        } else {
            Text::from(vec![plain_line("No panes discovered yet.")])
        });
    }

    None
}

fn sidebar_line(state: &AppState, theme: &Theme, entry: &VisibleTargetEntry) -> Line<'static> {
    let selected = state.selection.as_ref() == Some(&entry.target);
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
            .flash_label_for_target(&entry.target)
            .map(|label| format!("[{label}] "))
            .unwrap_or_default();
        if !flash_label.is_empty() {
            spans.push(Span::styled(flash_label, theme.warning_border));
        }
    }

    match &entry.sidebar {
        SidebarRowKind::Session {
            name,
            collapsed,
            rank,
            visible_windows,
            visible_panes,
            harnesses,
        } => {
            let marks = render_harness_marks(theme, harnesses);
            spans.push(Span::styled(
                format!(
                    "{} ",
                    if *collapsed {
                        theme.glyphs.session_closed
                    } else {
                        theme.glyphs.session_open
                    }
                ),
                attention_style_from_rank(theme, *rank),
            ));
            spans.push(Span::styled(
                name.clone(),
                if selected {
                    theme.selected
                } else {
                    theme.emphasis
                },
            ));
            spans.push(Span::styled(
                format!("  {}w/{}p", visible_windows, visible_panes),
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
        SidebarRowKind::Window {
            name,
            rank,
            visible_panes,
            harnesses,
        } => {
            let marks = render_harness_marks(theme, harnesses);
            spans.push(Span::styled("  ", theme.muted));
            spans.push(Span::styled("· ", attention_style_from_rank(theme, *rank)));
            spans.push(Span::styled(
                name.clone(),
                if selected { theme.selected } else { theme.base },
            ));
            spans.push(Span::styled(
                format!(" {}p", visible_panes),
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
        SidebarRowKind::Pane {
            navigation_title,
            status,
            harness,
            is_agent,
        } => {
            spans.push(Span::styled("    ", theme.muted));
            spans.push(Span::styled(
                format!("{} ", status_symbol(theme, *status, *is_agent)),
                status_style(theme, *status, *is_agent),
            ));
            spans.push(Span::styled(
                format!("{} ", harness_badge(theme, *harness)),
                if *is_agent {
                    status_style(theme, *status, true)
                } else {
                    theme.muted
                },
            ));
            spans.push(Span::styled(
                navigation_title.clone(),
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

    if state.startup_loading && state.inventory.sessions.is_empty() {
        return Text::from(vec![
            section_line("Loading tmux inventory", theme),
            plain_line(""),
            muted_line(
                "Sessions and panes will appear after the first background refresh.",
                theme,
            ),
            muted_line(
                "Visible and selected previews are captured first after the initial paint.",
                theme,
            ),
        ]);
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

    if let Some(age_ms) = state.startup_cache_age_ms {
        lines.push(section_line("Startup", theme));
        lines.push(plain_line(format!(
            "Cached snapshot: {}",
            humanize_age_ms(age_ms)
        )));
        lines.push(muted_line(
            if state.startup_loading {
                "Showing the last snapshot while live tmux inventory refreshes in the background."
            } else {
                "Still showing the cached snapshot because live tmux has not replaced it yet."
            },
            theme,
        ));
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
                lines.push(preview_source_line(theme, pane));
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
                let cached_visible_panes =
                    state
                        .selected_visible_entry()
                        .and_then(|entry| match &entry.sidebar {
                            SidebarRowKind::Window { visible_panes, .. } => Some(*visible_panes),
                            _ => None,
                        });
                lines.push(Line::from(vec![Span::styled(
                    format!("Window {}", window.navigation_title()),
                    theme.emphasis,
                )]));
                lines.push(plain_line(format!(
                    "Visible panes: {}",
                    cached_visible_panes.unwrap_or_else(|| window
                        .visible_panes(&state.filters, state.sort_mode)
                        .len())
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
                let cached_counts =
                    state
                        .selected_visible_entry()
                        .and_then(|entry| match &entry.sidebar {
                            SidebarRowKind::Session {
                                visible_windows,
                                visible_panes,
                                ..
                            } => Some((*visible_windows, *visible_panes)),
                            _ => None,
                        });
                lines.push(Line::from(vec![Span::styled(
                    format!("Session {}", session.name),
                    theme.emphasis,
                )]));
                lines.push(plain_line(format!(
                    "Visible: {} windows / {} panes",
                    cached_counts
                        .map(|(windows, _)| windows)
                        .unwrap_or_else(|| {
                            session
                                .visible_windows(&state.filters, state.sort_mode)
                                .len()
                        }),
                    cached_counts.map(|(_, panes)| panes).unwrap_or_else(|| {
                        session
                            .visible_windows(&state.filters, state.sort_mode)
                            .iter()
                            .map(|window| {
                                window.visible_panes(&state.filters, state.sort_mode).len()
                            })
                            .sum::<usize>()
                    })
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
            let preview_lines = preview_excerpt(&pane.preview, preview_line_limit(layout_mode));
            if preview_lines.is_empty() {
                lines.push(muted_line(pane.preview_provenance.detail(), theme));
            }
            for line in preview_lines {
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

    if state.startup_loading && state.inventory.sessions.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Loading tmux inventory before compose is available.",
            theme.emphasis,
        )]));
        lines.push(muted_line(
            "Foreman will enable compose as soon as the first refresh resolves actionable panes.",
            theme,
        ));
        return Text::from(lines);
    }

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
        Mode::FlashNavigate => {
            let mode_name = state
                .flash
                .as_ref()
                .map(|flash| match flash.kind {
                    crate::app::FlashNavigateKind::Jump => "jump",
                    crate::app::FlashNavigateKind::JumpAndFocus => "jump+focus",
                })
                .unwrap_or("jump");
            let typed = state
                .flash
                .as_ref()
                .map(|flash| flash.draft.text.as_str())
                .filter(|typed| !typed.is_empty())
                .unwrap_or("type label");
            vec![format!(
                "Flash inline: {typed}{sep}mode {mode_name}{sep}labels stay visible in the list{sep}Esc cancel"
            )]
        }
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
        section_line("Legend", theme),
        muted_line(
            format!(
                "Tree: {} expanded session  {} collapsed session",
                theme.glyphs.session_open, theme.glyphs.session_closed
            ),
            theme,
        ),
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
                "Compat: {} Gemini  {} OpenCode  {} shell/plain",
                theme.glyphs.gemini, theme.glyphs.opencode, theme.glyphs.shell
            ),
            theme,
        ),
        plain_line(""),
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
        muted_line("Flash labels render inline; there is no blocking popup.", theme),
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
            "o cycles stable and attention->recent. t themes. m mutes. n changes notification profile.",
            theme,
        ),
        plain_line(""),
        section_line("Debug", theme),
        muted_line(
            "Run foreman --debug and inspect latest.log for render_frame, inventory_tmux, inventory_native, and move-selection timings.",
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
        return Vec::new();
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

fn render_harness_marks(theme: &Theme, summary: &SidebarHarnessSummary) -> String {
    let mut marks = summary
        .harnesses
        .iter()
        .map(|harness| harness_badge(theme, Some(*harness)))
        .collect::<Vec<_>>();

    if summary.saw_shell && marks.is_empty() {
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
        preview_source_line(theme, pane),
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

fn preview_source_line(theme: &Theme, pane: &Pane) -> Line<'static> {
    let style = match pane.preview_provenance {
        crate::app::PreviewProvenance::PendingCapture => theme.warning_border,
        crate::app::PreviewProvenance::CaptureFailed => theme.error,
        crate::app::PreviewProvenance::Captured | crate::app::PreviewProvenance::ReusedCached => {
            theme.muted
        }
    };
    Line::from(vec![
        Span::styled("Preview source: ", theme.muted),
        Span::styled(pane.preview_provenance.label(), style),
    ])
}

fn humanize_age_ms(age_ms: u64) -> String {
    if age_ms < 1_000 {
        "<1s ago".to_string()
    } else if age_ms < 60_000 {
        format!("{}s ago", age_ms / 1_000)
    } else {
        format!("{}m ago", age_ms / 60_000)
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
    let sort_hint = format!("Sort {}", state.sort_label());
    let secondary = match layout_mode {
        LayoutMode::Compact => format!("View h/o/t{sep}Help ?"),
        LayoutMode::Medium => {
            format!("Panels Tab or 1/2/3{sep}Find / s S{sep}{sort_hint}{sep}View h o t{sep}Help ?")
        }
        LayoutMode::Wide => {
            format!("Panels Tab or 1/2/3{sep}Find / s S{sep}{sort_hint}{sep}View h o t{sep}Alerts m n{sep}Help ?")
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
                format!("Details: j/k scroll{sep}f jump tmux{sep}i compose")
            } else {
                format!("Details: j/k scroll{sep}Tab sidebar{sep}Help ?")
            };
            let pr_actions = if state.selected_pull_request().is_some() {
                format!("PR p{sep}Open O{sep}Copy Y")
            } else {
                "PR p".to_string()
            };
            match layout_mode {
                LayoutMode::Compact => {
                    vec![format!(
                        "Details: j/k scroll{sep}f tmux{sep}p PR{sep}Help ?"
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
        PaneBuilder, PreviewProvenance, SearchState, SelectionTarget, SessionBuilder,
        WindowBuilder,
    };
    use crate::doctor::{DoctorArea, DoctorFinding, DoctorSeverity};
    use crate::services::pull_requests::{PullRequestData, PullRequestLookup, PullRequestStatus};
    use crate::services::system_stats::SystemStatsSnapshot;
    use crate::ui::theme::ThemeName;
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;
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

    fn tall_sidebar_state() -> AppState {
        let inventory = inventory([SessionBuilder::new("alpha").window({
            let mut window = WindowBuilder::new("alpha:agents").name("agents");
            for index in 0..12 {
                window = window.pane(
                    PaneBuilder::agent(format!("alpha:pane:{index}"), HarnessKind::ClaudeCode)
                        .title(format!("pane-{index}"))
                        .working_dir(format!("/tmp/pane-{index}"))
                        .status(AgentStatus::Working),
                );
            }
            window
        })]);

        let mut state = AppState::with_inventory(inventory);
        state.selection = Some(SelectionTarget::Pane("alpha:pane:8".into()));
        state.set_sidebar_viewport_rows(super::sidebar_viewport_rows_for_area(
            ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 20,
            },
        ) as usize);
        state.reconcile_sidebar_scroll();
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
    fn render_shows_loading_state_before_first_inventory_refresh() {
        let state = AppState {
            startup_loading: true,
            ..AppState::default()
        };
        let output = render_to_string(&state);

        assert!(output.contains("Loading tmux inventory"));
        assert!(output.contains("drawing immediately"));
        assert!(output.contains("before compose is available"));
    }

    #[test]
    fn render_surfaces_cached_startup_provenance() {
        let mut state = sample_state();
        state.startup_loading = true;
        state.startup_cache_age_ms = Some(4_200);

        let output = render_to_string(&state);

        assert!(output.contains("cached 4s ago"));
        assert!(output.contains("Cached snapshot: 4s ago"));
        assert!(output.contains("Showing the last snapshot"));
        assert!(output.contains("background"));
    }

    #[test]
    fn render_displays_help_overlay_with_extended_command_surface() {
        let mut state = sample_state();
        state.mode = Mode::Help;
        let output = render_to_string(&state);

        let legend_index = output.find("Legend").expect("legend should render");
        let right_now_index = output.find("Right now").expect("right now should render");
        assert!(legend_index < right_now_index);
        assert!(output.contains("Tree:"));
        assert!(output.contains("Right now"));
        assert!(output.contains("Focus: Sidebar"));
        assert!(output.contains("Source"));
        assert!(output.contains("Navigate"));
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
        assert!(output.contains("Preview source: captured"));
        assert!(output.contains("f jumps tmux to the target pane"));
    }

    #[test]
    fn render_preview_surfaces_status_provenance_for_selected_pane() {
        let state = sample_state();
        let output = render_to_string(&state);

        assert!(output.contains("Status source: native hook"));
        assert!(output.contains("high confidence"));
        assert!(output.contains("Preview source: captured"));
    }

    #[test]
    fn render_surfaces_degraded_preview_provenance_for_selected_pane() {
        let inventory = inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").name("agents").pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .title("M1-AFurrier")
                        .current_command("claude")
                        .working_dir("/tmp/alpha")
                        .preview("")
                        .preview_provenance(PreviewProvenance::CaptureFailed)
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
        state.rebuild_visible_state();

        let output = render_to_string(&state);

        assert!(output.contains("Preview source: capture failed"));
        assert!(output.contains("tmux capture failed on the latest refresh"));
    }

    #[test]
    fn render_surfaces_compound_sort_labels() {
        let mut state = sample_state();
        let recent_output = render_to_string(&state);
        assert!(recent_output.contains("View: stable"));
        assert!(recent_output.contains("Sort stable"));

        state.sort_mode = crate::app::SortMode::AttentionFirst;
        state.rebuild_visible_state();
        let attention_output = render_to_string(&state);
        assert!(attention_output.contains("View: attention->recent"));
        assert!(attention_output.contains("Sort attention->recent"));
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
    fn render_preview_clamps_large_scroll_offsets() {
        let mut state = sample_state();
        state.focus = Focus::Preview;
        state.preview_scroll = u16::MAX;
        let output = render_to_string_at(&state, ThemeName::Catppuccin, 100, 20);

        assert!(output.contains("Applying patch"));
    }

    #[test]
    fn render_displays_search_overlay_and_match_count() {
        let mut state = sample_state();
        state.mode = Mode::Search;
        let mut search = SearchState::new(state.selection.clone());
        search.draft.text = "codex".to_string();
        state.search = Some(search);
        state.selection = Some(SelectionTarget::Pane("beta:codex".into()));
        state.rebuild_visible_state();
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

        assert!(output.contains("Flash inline"));
        assert!(output.contains("labels stay visible in the list"));
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
    fn render_sidebar_virtualizes_rows_in_small_viewport() {
        let state = tall_sidebar_state();
        let output = render_to_string_at(&state, ThemeName::Catppuccin, 100, 20);

        assert!(output.contains("pane-8"));
        assert!(!output.contains("pane-0"));
        assert!(!output.contains("pane-1"));
    }

    #[test]
    fn render_footer_uses_labeled_control_groups() {
        let state = sample_state();
        let output = render_to_string(&state);

        assert!(output.contains("Keys • Sidebar") || output.contains("Keys | Sidebar"));
        assert!(output.contains("Sidebar: j/k move"));
        assert!(output.contains("Enter use row"));
        assert!(output.contains("Sort stable"));
        assert!(output.contains("Find / s S"));
        assert!(output.contains("Help ?"));
    }

    #[test]
    fn render_help_explains_sort_cycle() {
        let mut state = sample_state();
        state.mode = Mode::Help;
        state.help_scroll = 24;
        let output = render_to_string_at(&state, ThemeName::Catppuccin, 140, 36);

        assert!(output.contains("o cycles"));
    }

    #[test]
    fn render_help_surfaces_debug_trace_hint() {
        let mut state = sample_state();
        state.mode = Mode::Help;
        let theme = ThemeName::Catppuccin.resolve();
        let help_text = super::help_lines(&state, &theme)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(help_text.contains("Debug"));
        assert!(help_text.contains("Run foreman --debug"));
        assert!(help_text.contains("latest.log"));
    }

    #[test]
    fn terminal_theme_uses_warm_working_and_semantic_idle_error_colors() {
        let theme = ThemeName::Terminal.resolve();

        assert_eq!(theme.working.fg, Some(Color::Yellow));
        assert_eq!(theme.idle.fg, Some(Color::Green));
        assert_eq!(theme.error.fg, Some(Color::Red));
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
