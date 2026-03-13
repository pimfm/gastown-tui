use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, LineGauge, List, ListItem, Padding,
        Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, Tabs, Wrap,
    },
    Frame,
};

use crate::app::{App, TAB_NAMES};
use crate::gastown::{AgentStatus, BeadStatus, ConvoyStatus, Severity};

// ── Color palette ────────────────────────────────────────────────────

const BG: Color = Color::Rgb(22, 22, 30);
const SURFACE: Color = Color::Rgb(30, 30, 42);
const BORDER: Color = Color::Rgb(60, 60, 80);
const ACCENT: Color = Color::Rgb(130, 170, 255);
const ACCENT_DIM: Color = Color::Rgb(80, 110, 180);
const GREEN: Color = Color::Rgb(120, 220, 140);
const YELLOW: Color = Color::Rgb(240, 200, 80);
const RED: Color = Color::Rgb(240, 100, 100);
const ORANGE: Color = Color::Rgb(240, 160, 80);
const MUTED: Color = Color::Rgb(100, 100, 120);
const TEXT: Color = Color::Rgb(200, 200, 220);
const TEXT_BRIGHT: Color = Color::Rgb(240, 240, 255);
const CYAN: Color = Color::Rgb(100, 220, 230);
const PURPLE: Color = Color::Rgb(180, 140, 255);

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::default().bg(BG)), area);

    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(1),
    ])
    .areas(area);

    draw_header(frame, app, header_area);
    match app.tab {
        0 => draw_dashboard(frame, app, body_area),
        1 => draw_agents(frame, app, body_area),
        2 => draw_convoys(frame, app, body_area),
        3 => draw_beads(frame, app, body_area),
        4 => draw_repos(frame, app, body_area),
        _ => {}
    }
    draw_footer(frame, app, footer_area);

    if app.show_spawn_dialog {
        draw_spawn_dialog(frame, app, area);
    }
}

// ── Header ───────────────────────────────────────────────────────────

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let [title_area, tabs_area] =
        Layout::horizontal([Constraint::Length(20), Constraint::Min(40)]).areas(area);

    // Animated spinner
    let spinner = ['◐', '◓', '◑', '◒'];
    let spin_char = spinner[(app.tick as usize / 2) % spinner.len()];

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {spin_char} "),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "GAS TOWN",
            Style::default()
                .fg(TEXT_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" TUI", Style::default().fg(ACCENT_DIM)),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(BORDER))
            .border_type(BorderType::Rounded),
    )
    .alignment(Alignment::Left);
    frame.render_widget(title, title_area);

    let tab_titles: Vec<Line> = TAB_NAMES
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let num = format!("{}", i + 1);
            Line::from(vec![
                Span::styled(
                    num,
                    Style::default().fg(ACCENT_DIM).add_modifier(Modifier::DIM),
                ),
                Span::raw(":"),
                Span::styled(t.to_string(), Style::default().fg(TEXT)),
            ])
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .select(app.tab)
        .highlight_style(
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )
        .divider(Span::styled(" │ ", Style::default().fg(BORDER)))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(BORDER))
                .border_type(BorderType::Rounded),
        );
    frame.render_widget(tabs, tabs_area);
}

// ── Footer ───────────────────────────────────────────────────────────

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let status = if app.gt_available {
        Span::styled(" GT ✓ ", Style::default().fg(GREEN).bg(Color::Rgb(30, 50, 30)))
    } else {
        Span::styled(
            " GT: demo mode ",
            Style::default().fg(YELLOW).bg(Color::Rgb(50, 45, 20)),
        )
    };

    let help = if app.filter_active {
        Line::from(vec![
            Span::styled(" / ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(&app.filter_text, Style::default().fg(TEXT_BRIGHT)),
            Span::styled("█", Style::default().fg(ACCENT)),
            Span::styled("  Esc:close  Enter:apply", Style::default().fg(MUTED)),
        ])
    } else {
        Line::from(vec![
            status,
            Span::raw(" "),
            Span::styled("q", Style::default().fg(ACCENT)),
            Span::styled(":quit ", Style::default().fg(MUTED)),
            Span::styled("Tab", Style::default().fg(ACCENT)),
            Span::styled(":switch ", Style::default().fg(MUTED)),
            Span::styled("j/k", Style::default().fg(ACCENT)),
            Span::styled(":scroll ", Style::default().fg(MUTED)),
            Span::styled("/", Style::default().fg(ACCENT)),
            Span::styled(":filter ", Style::default().fg(MUTED)),
            Span::styled("s", Style::default().fg(ACCENT)),
            Span::styled(":spawn ", Style::default().fg(MUTED)),
            Span::styled("r", Style::default().fg(ACCENT)),
            Span::styled(":refresh", Style::default().fg(MUTED)),
        ])
    };

    let footer = Paragraph::new(help).style(Style::default().bg(SURFACE));
    frame.render_widget(footer, area);
}

// ── Dashboard ────────────────────────────────────────────────────────

fn draw_dashboard(frame: &mut Frame, app: &mut App, area: Rect) {
    let [top, mid, bottom] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(12),
        Constraint::Min(6),
    ])
    .areas(area);

    // ── Stats cards ──
    let [card1, card2, card3, card4] = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .areas(top);

    draw_stat_card(
        frame,
        card1,
        "Agents Active",
        &app.agents_active.to_string(),
        &format!("/{}", app.agents.len()),
        GREEN,
    );
    draw_stat_card(
        frame,
        card2,
        "Beads Done",
        &app.total_beads_done.to_string(),
        &format!(
            "/{}",
            app.total_beads_done
                + app.total_beads_open
                + app.total_beads_progress
                + app.total_beads_blocked
        ),
        CYAN,
    );
    draw_stat_card(
        frame,
        card3,
        "Active Convoys",
        &app.active_convoys().len().to_string(),
        &format!("/{}", app.convoys.len()),
        PURPLE,
    );
    draw_stat_card(
        frame,
        card4,
        "Stuck/Blocked",
        &format!(
            "{}/{}",
            app.agents_stuck, app.total_beads_blocked
        ),
        "agents/beads",
        if app.agents_stuck > 0 || app.total_beads_blocked > 0 {
            RED
        } else {
            GREEN
        },
    );

    // ── Convoy progress bars ──
    let [convoy_area, agent_mini_area] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(mid);

    draw_convoy_progress(frame, app, convoy_area);
    draw_agent_status_mini(frame, app, agent_mini_area);

    // ── Activity feed ──
    draw_feed(frame, app, bottom);
}

fn draw_stat_card(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    suffix: &str,
    color: Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [label_area, value_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(2)]).areas(inner);

    let label_w = Paragraph::new(Span::styled(
        label,
        Style::default().fg(MUTED).add_modifier(Modifier::DIM),
    ));
    frame.render_widget(label_w, label_area);

    let value_w = Paragraph::new(Line::from(vec![
        Span::styled(
            value,
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {suffix}"),
            Style::default().fg(MUTED),
        ),
    ]));
    frame.render_widget(value_w, value_area);
}

fn draw_convoy_progress(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Convoy Progress ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let active = app.active_convoys();
    let constraints: Vec<Constraint> = active.iter().map(|_| Constraint::Length(2)).collect();
    if constraints.is_empty() {
        let empty = Paragraph::new(Span::styled("No active convoys", Style::default().fg(MUTED)));
        frame.render_widget(empty, inner);
        return;
    }
    let rows = Layout::vertical(constraints).split(inner);

    for (i, convoy) in active.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let [label_area, gauge_area] =
            Layout::horizontal([Constraint::Length(22), Constraint::Min(10)]).areas(rows[i]);

        let truncated_name = if convoy.name.len() > 18 {
            format!("{}…", &convoy.name[..17])
        } else {
            convoy.name.clone()
        };

        let label = Paragraph::new(Span::styled(truncated_name, Style::default().fg(TEXT)));
        frame.render_widget(label, label_area);

        let color = match convoy.progress() {
            p if p >= 0.9 => GREEN,
            p if p >= 0.5 => CYAN,
            p if p >= 0.2 => YELLOW,
            _ => ORANGE,
        };

        let pct = (convoy.progress() * 100.0) as u16;
        let gauge = LineGauge::default()
            .filled_style(Style::default().fg(color))
            .unfilled_style(Style::default().fg(Color::Rgb(40, 40, 55)))
            .line_set(symbols::line::THICK)
            .label(Span::styled(
                format!(
                    "{}/{} ({}%)",
                    convoy.completed_beads, convoy.total_beads, pct
                ),
                Style::default().fg(TEXT),
            ))
            .ratio(convoy.progress());
        frame.render_widget(gauge, gauge_area);
    }
}

fn draw_agent_status_mini(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Agent Overview ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render a mini grid of agent status dots
    let cols = (inner.width as usize).max(1);
    let agents = &app.agents;
    let mut lines: Vec<Line> = Vec::new();
    let mut spans: Vec<Span> = Vec::new();

    for (i, agent) in agents.iter().enumerate() {
        let (sym, color) = match agent.status {
            AgentStatus::Active => ("●", GREEN),
            AgentStatus::Idle => ("○", MUTED),
            AgentStatus::Stuck => ("◆", YELLOW),
            AgentStatus::Offline => ("·", Color::Rgb(50, 50, 60)),
        };
        spans.push(Span::styled(format!("{sym} "), Style::default().fg(color)));

        if (i + 1) % (cols / 2).max(1) == 0 || i == agents.len() - 1 {
            lines.push(Line::from(std::mem::take(&mut spans)));
        }
    }

    // Legend
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("● ", Style::default().fg(GREEN)),
        Span::styled("active  ", Style::default().fg(MUTED)),
        Span::styled("○ ", Style::default().fg(MUTED)),
        Span::styled("idle  ", Style::default().fg(MUTED)),
        Span::styled("◆ ", Style::default().fg(YELLOW)),
        Span::styled("stuck  ", Style::default().fg(MUTED)),
        Span::styled("· ", Style::default().fg(Color::Rgb(50, 50, 60))),
        Span::styled("offline", Style::default().fg(MUTED)),
    ]));

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}

fn draw_feed(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Activity Feed ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = app
        .feed
        .iter()
        .map(|entry| {
            let sev_color = match entry.severity {
                Severity::Info => ACCENT_DIM,
                Severity::Warning => YELLOW,
                Severity::Error => RED,
                Severity::Success => GREEN,
            };
            let sev_sym = match entry.severity {
                Severity::Info => "ℹ",
                Severity::Warning => "⚠",
                Severity::Error => "✗",
                Severity::Success => "✓",
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", entry.timestamp),
                    Style::default().fg(MUTED),
                ),
                Span::styled(format!("{sev_sym} "), Style::default().fg(sev_color)),
                Span::styled(
                    format!("{:<12} ", entry.agent),
                    Style::default()
                        .fg(CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<10} ", entry.action),
                    Style::default().fg(sev_color),
                ),
                Span::styled(&entry.detail, Style::default().fg(TEXT)),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

// ── Agents tab ───────────────────────────────────────────────────────

fn draw_agents(frame: &mut Frame, app: &mut App, area: Rect) {
    let count = app.filtered_agents().len();
    app.max_scroll = count.saturating_sub(1);
    let agents = app.filtered_agents();

    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Runtime").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Rig").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Current Bead").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Done").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Uptime").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = agents
        .iter()
        .skip(app.scroll)
        .map(|agent| {
            let status_color = match agent.status {
                AgentStatus::Active => GREEN,
                AgentStatus::Idle => MUTED,
                AgentStatus::Stuck => YELLOW,
                AgentStatus::Offline => RED,
            };
            let uptime = format_duration(agent.uptime_secs);
            Row::new(vec![
                Cell::from(agent.name.clone()).style(Style::default().fg(TEXT_BRIGHT)),
                Cell::from(agent.status.label()).style(
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(agent.runtime.clone()).style(Style::default().fg(PURPLE)),
                Cell::from(agent.rig.clone()).style(Style::default().fg(CYAN)),
                Cell::from(
                    agent
                        .current_bead
                        .clone()
                        .unwrap_or_else(|| "—".into()),
                )
                .style(Style::default().fg(ACCENT_DIM)),
                Cell::from(agent.beads_completed.to_string())
                    .style(Style::default().fg(GREEN)),
                Cell::from(uptime).style(Style::default().fg(MUTED)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(14),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(16),
        Constraint::Length(14),
        Constraint::Length(6),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header.style(Style::default().bg(SURFACE)))
        .row_highlight_style(Style::default().bg(Color::Rgb(40, 40, 60)))
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Agents ({}) ", agents.len()),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER))
                .style(Style::default().bg(SURFACE)),
        );

    frame.render_widget(table, area);

    // Scrollbar
    let sb_area = area.inner(Margin {
        vertical: 1,
        horizontal: 0,
    });
    let mut sb_state =
        ScrollbarState::new(agents.len()).position(app.scroll);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(ACCENT_DIM))
            .track_style(Style::default().fg(Color::Rgb(35, 35, 50))),
        sb_area,
        &mut sb_state,
    );
}

// ── Convoys tab ──────────────────────────────────────────────────────

fn draw_convoys(frame: &mut Frame, app: &mut App, area: Rect) {
    let count = app.filtered_convoys().len();
    app.max_scroll = count.saturating_sub(1);
    let convoys = app.filtered_convoys();

    let block = Block::default()
        .title(Span::styled(
            format!(" Convoys ({}) ", convoys.len()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Each convoy gets 4 lines
    let constraints: Vec<Constraint> = convoys
        .iter()
        .skip(app.scroll)
        .take(inner.height as usize / 4)
        .map(|_| Constraint::Length(4))
        .collect();
    if constraints.is_empty() {
        let empty = Paragraph::new(Span::styled("No convoys found", Style::default().fg(MUTED)));
        frame.render_widget(empty, inner);
        return;
    }
    let rows = Layout::vertical(constraints).split(inner);

    for (i, convoy) in convoys.iter().skip(app.scroll).enumerate() {
        if i >= rows.len() {
            break;
        }
        let row = rows[i];
        let [line1, line2, line3, _sep] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(row);

        let status_color = match convoy.status {
            ConvoyStatus::Active => GREEN,
            ConvoyStatus::Completed => CYAN,
            ConvoyStatus::Paused => YELLOW,
            ConvoyStatus::Blocked => RED,
        };

        // Line 1: ID, name, status
        let header_line = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} ", convoy.id),
                Style::default().fg(ACCENT_DIM),
            ),
            Span::styled(
                &convoy.name,
                Style::default()
                    .fg(TEXT_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                convoy.status.label(),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  [{} agents]",
                    convoy.agents_assigned.len()
                ),
                Style::default().fg(MUTED),
            ),
        ]));
        frame.render_widget(header_line, line1);

        // Line 2: Progress gauge
        let pct = (convoy.progress() * 100.0) as u16;
        let gauge_color = match convoy.progress() {
            p if p >= 1.0 => GREEN,
            p if p >= 0.7 => CYAN,
            p if p >= 0.3 => YELLOW,
            _ => ORANGE,
        };
        let gauge = LineGauge::default()
            .filled_style(Style::default().fg(gauge_color))
            .unfilled_style(Style::default().fg(Color::Rgb(40, 40, 55)))
            .line_set(symbols::line::THICK)
            .label(Span::styled(
                format!(
                    "  {} done / {} in progress / {} remaining  ({}%)",
                    convoy.completed_beads,
                    convoy.in_progress_beads,
                    convoy.total_beads - convoy.completed_beads - convoy.in_progress_beads,
                    pct,
                ),
                Style::default().fg(TEXT),
            ))
            .ratio(convoy.progress());
        frame.render_widget(gauge, line2);

        // Line 3: Assigned agents
        let agent_spans: Vec<Span> = convoy
            .agents_assigned
            .iter()
            .flat_map(|a| {
                vec![
                    Span::styled(a, Style::default().fg(CYAN)),
                    Span::styled("  ", Style::default()),
                ]
            })
            .collect();
        let mut full_spans = vec![Span::styled("  Agents: ", Style::default().fg(MUTED))];
        full_spans.extend(agent_spans);
        frame.render_widget(Paragraph::new(Line::from(full_spans)), line3);
    }
}

// ── Beads tab ────────────────────────────────────────────────────────

fn draw_beads(frame: &mut Frame, app: &mut App, area: Rect) {
    let count = app.filtered_beads().len();
    app.max_scroll = count.saturating_sub(1);
    let beads = app.filtered_beads();

    let header = Row::new(vec![
        Cell::from("ID").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Title").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Rig").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Assigned To").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Convoy").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = beads
        .iter()
        .skip(app.scroll)
        .map(|bead| {
            let status_color = match bead.status {
                BeadStatus::Open => MUTED,
                BeadStatus::InProgress => CYAN,
                BeadStatus::Done => GREEN,
                BeadStatus::Blocked => RED,
            };
            Row::new(vec![
                Cell::from(bead.id.clone()).style(Style::default().fg(ACCENT_DIM)),
                Cell::from(bead.status.label()).style(
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(bead.title.clone()).style(Style::default().fg(TEXT)),
                Cell::from(bead.rig.clone()).style(Style::default().fg(PURPLE)),
                Cell::from(
                    bead.assigned_to
                        .clone()
                        .unwrap_or_else(|| "—".into()),
                )
                .style(Style::default().fg(CYAN)),
                Cell::from(
                    bead.convoy_id
                        .clone()
                        .unwrap_or_else(|| "—".into()),
                )
                .style(Style::default().fg(MUTED)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(9),
        Constraint::Min(30),
        Constraint::Length(16),
        Constraint::Length(14),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header.style(Style::default().bg(SURFACE)))
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Beads ({}) ", beads.len()),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER))
                .style(Style::default().bg(SURFACE)),
        );

    frame.render_widget(table, area);

    let sb_area = area.inner(Margin {
        vertical: 1,
        horizontal: 0,
    });
    let mut sb_state =
        ScrollbarState::new(beads.len()).position(app.scroll);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(ACCENT_DIM))
            .track_style(Style::default().fg(Color::Rgb(35, 35, 50))),
        sb_area,
        &mut sb_state,
    );
}

// ── Repos tab ────────────────────────────────────────────────────────

fn draw_repos(frame: &mut Frame, app: &mut App, area: Rect) {
    let count = app.filtered_repos().len();
    app.max_scroll = count.saturating_sub(1);
    let repos = app.filtered_repos();

    let header = Row::new(vec![
        Cell::from("Repository").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Branch").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("↑↓").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Last Commit").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Path").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = repos
        .iter()
        .skip(app.scroll)
        .map(|repo| {
            let status_text = if repo.dirty { "modified" } else { "clean" };
            let status_color = if repo.dirty { YELLOW } else { GREEN };

            let sync = if repo.ahead > 0 && repo.behind > 0 {
                format!("↑{}↓{}", repo.ahead, repo.behind)
            } else if repo.ahead > 0 {
                format!("↑{}", repo.ahead)
            } else if repo.behind > 0 {
                format!("↓{}", repo.behind)
            } else {
                "synced".into()
            };
            let sync_color = if repo.ahead > 0 || repo.behind > 0 {
                ORANGE
            } else {
                MUTED
            };

            let path_str = repo
                .path
                .to_string_lossy()
                .replace(&dirs::home_dir().unwrap_or_default().to_string_lossy().to_string(), "~");

            let commit = if repo.last_commit.len() > 40 {
                format!("{}…", &repo.last_commit[..39])
            } else {
                repo.last_commit.clone()
            };

            Row::new(vec![
                Cell::from(repo.name.clone()).style(
                    Style::default()
                        .fg(TEXT_BRIGHT)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(repo.branch.clone()).style(Style::default().fg(PURPLE)),
                Cell::from(status_text).style(
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(sync).style(Style::default().fg(sync_color)),
                Cell::from(commit).style(Style::default().fg(MUTED)),
                Cell::from(path_str).style(Style::default().fg(ACCENT_DIM)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(18),
        Constraint::Length(14),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Min(30),
        Constraint::Length(30),
    ];

    let table = Table::new(rows, widths)
        .header(header.style(Style::default().bg(SURFACE)))
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Repositories ({}) ", repos.len()),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER))
                .style(Style::default().bg(SURFACE)),
        );

    frame.render_widget(table, area);

    let sb_area = area.inner(Margin {
        vertical: 1,
        horizontal: 0,
    });
    let mut sb_state =
        ScrollbarState::new(repos.len()).position(app.scroll);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(ACCENT_DIM))
            .track_style(Style::default().fg(Color::Rgb(35, 35, 50))),
        sb_area,
        &mut sb_state,
    );
}

// ── Spawn dialog ─────────────────────────────────────────────────────

fn draw_spawn_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let dialog_width = 50u16.min(area.width.saturating_sub(4));
    let dialog_height = 14u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Dim background
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(Span::styled(
            " Spawn Agent ",
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(SURFACE));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let [field1, field2, field3, _gap, help_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .areas(inner);

    let field_style = |active: bool| {
        if active {
            Style::default().fg(TEXT_BRIGHT)
        } else {
            Style::default().fg(MUTED)
        }
    };
    let label_style = |active: bool| {
        if active {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED)
        }
    };

    // Rig field
    let rig_text = if app.spawn_rig.is_empty() && app.spawn_field == 0 {
        "type rig name…"
    } else if app.spawn_rig.is_empty() {
        ""
    } else {
        &app.spawn_rig
    };
    let rig_line = Line::from(vec![
        Span::styled(" Rig:     ", label_style(app.spawn_field == 0)),
        Span::styled(rig_text, field_style(app.spawn_field == 0)),
        if app.spawn_field == 0 {
            Span::styled("█", Style::default().fg(ACCENT))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(rig_line), field1);

    // Runtime selector
    let runtime_line = Line::from(vec![
        Span::styled(" Runtime: ", label_style(app.spawn_field == 1)),
        Span::styled(
            format!("◀ {} ▶", app.spawn_runtime()),
            field_style(app.spawn_field == 1),
        ),
    ]);
    frame.render_widget(Paragraph::new(runtime_line), field2);

    // Task field
    let task_text = if app.spawn_task.is_empty() && app.spawn_field == 2 {
        "describe the task…"
    } else if app.spawn_task.is_empty() {
        ""
    } else {
        &app.spawn_task
    };
    let task_line = Line::from(vec![
        Span::styled(" Task:    ", label_style(app.spawn_field == 2)),
        Span::styled(task_text, field_style(app.spawn_field == 2)),
        if app.spawn_field == 2 {
            Span::styled("█", Style::default().fg(ACCENT))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(task_line), field3);

    // Help
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" Tab", Style::default().fg(ACCENT)),
        Span::styled(":next field  ", Style::default().fg(MUTED)),
        Span::styled("↑↓", Style::default().fg(ACCENT)),
        Span::styled(":runtime  ", Style::default().fg(MUTED)),
        Span::styled("Enter", Style::default().fg(ACCENT)),
        Span::styled(":spawn  ", Style::default().fg(MUTED)),
        Span::styled("Esc", Style::default().fg(ACCENT)),
        Span::styled(":cancel", Style::default().fg(MUTED)),
    ]));
    frame.render_widget(help, help_area);
}

// ── Helpers ──────────────────────────────────────────────────────────

fn format_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m")
    }
}
