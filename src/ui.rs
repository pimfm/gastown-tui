use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, LineGauge, List, ListItem, Padding, Paragraph,
        Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, Tabs, Wrap,
    },
    Frame,
};

use crate::app::{App, TAB_NAMES};

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
        Layout::horizontal([Constraint::Length(22), Constraint::Min(40)]).areas(area);

    let spinner = ['\u{25D0}', '\u{25D3}', '\u{25D1}', '\u{25D2}'];
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
    );
    frame.render_widget(title, title_area);

    let tab_titles: Vec<Line> = TAB_NAMES
        .iter()
        .enumerate()
        .map(|(i, t)| {
            Line::from(vec![
                Span::styled(
                    format!("{}", i + 1),
                    Style::default()
                        .fg(ACCENT_DIM)
                        .add_modifier(Modifier::DIM),
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
        .divider(Span::styled(" | ", Style::default().fg(BORDER)))
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
    let api_status = if app.connected {
        Span::styled(
            " API OK ",
            Style::default().fg(GREEN).bg(Color::Rgb(30, 50, 30)),
        )
    } else {
        Span::styled(
            " API -- ",
            Style::default().fg(RED).bg(Color::Rgb(50, 30, 30)),
        )
    };

    let dolt_status = if app.infra.dolt_running {
        Span::styled("Dolt OK ", Style::default().fg(GREEN))
    } else {
        Span::styled("Dolt -- ", Style::default().fg(MUTED))
    };

    let help = if app.filter_active {
        Line::from(vec![
            Span::styled(
                " / ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(&app.filter_text, Style::default().fg(TEXT_BRIGHT)),
            Span::styled("\u{2588}", Style::default().fg(ACCENT)),
            Span::styled("  Esc:close  Enter:apply", Style::default().fg(MUTED)),
        ])
    } else if let Some(ref msg) = app.status_msg {
        Line::from(vec![
            Span::styled(format!(" {msg} "), Style::default().fg(YELLOW)),
            Span::styled("  Esc:dismiss", Style::default().fg(MUTED)),
        ])
    } else {
        Line::from(vec![
            api_status,
            Span::raw(" "),
            dolt_status,
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
        Constraint::Length(14),
        Constraint::Min(6),
    ])
    .areas(area);

    // ── Stat cards ──
    let [card1, card2, card3, card4, card5] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .areas(top);

    draw_stat_card(
        frame,
        card1,
        "Rigs",
        &app.stats.rigs.to_string(),
        "registered",
        CYAN,
    );
    draw_stat_card(
        frame,
        card2,
        "Agents",
        &format!("{}/{}", app.stats.agents_running, app.stats.agents_total),
        "running",
        if app.stats.agents_running > 0 {
            GREEN
        } else {
            MUTED
        },
    );
    draw_stat_card(
        frame,
        card3,
        "Polecats",
        &app.stats.polecats.to_string(),
        &format!("{} hooks", app.stats.active_hooks),
        PURPLE,
    );

    draw_stat_card(
        frame,
        card4,
        "Beads",
        &format!("{}/{}", app.stats.beads_in_progress, app.stats.beads_total),
        &format!("{} open", app.stats.beads_open),
        ACCENT,
    );

    draw_stat_card(
        frame,
        card5,
        "Mail",
        &app.stats.unread_mail.to_string(),
        "unread",
        if app.stats.unread_mail > 0 {
            YELLOW
        } else {
            MUTED
        },
    );

    // ── Rig overview + infrastructure ──
    let [rigs_area, infra_area] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(mid);

    draw_rig_overview(frame, app, rigs_area);
    draw_infra_status(frame, app, infra_area);

    // ── Recent beads ──
    draw_recent_beads(frame, app, bottom);
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

    frame.render_widget(
        Paragraph::new(Span::styled(label, Style::default().fg(MUTED))),
        label_area,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                value,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {suffix}"), Style::default().fg(MUTED)),
        ])),
        value_area,
    );
}

fn draw_rig_overview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Rigs ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.rigs.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No rigs registered",
                Style::default().fg(MUTED),
            )),
            inner,
        );
        return;
    }

    let header = Row::new(vec![
        Cell::from("Rig").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Polecats").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Crew").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Wit").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Ref").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Hooks").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = app
        .rigs
        .iter()
        .map(|rig| {
            Row::new(vec![
                Cell::from(rig.name.clone())
                    .style(Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD)),
                Cell::from(rig.polecats.to_string()).style(
                    Style::default().fg(if rig.polecats > 0 { GREEN } else { MUTED }),
                ),
                Cell::from(rig.crews.to_string()).style(Style::default().fg(MUTED)),
                Cell::from(if rig.has_witness { "Y" } else { "N" }).style(
                    Style::default().fg(if rig.has_witness { GREEN } else { MUTED }),
                ),
                Cell::from(if rig.has_refinery { "Y" } else { "N" }).style(
                    Style::default().fg(if rig.has_refinery { GREEN } else { MUTED }),
                ),
                Cell::from(format!("{}/{}", rig.hooks_active, rig.hooks_total)).style(
                    Style::default().fg(if rig.hooks_active > 0 { YELLOW } else { MUTED }),
                ),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(16),
        Constraint::Length(9),
        Constraint::Length(6),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);
}

fn draw_infra_status(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Infrastructure ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let status_line = |name: &str, running: bool, extra: &str| -> Line {
        let (sym, color) = if running {
            ("\u{25CF}", GREEN)
        } else {
            ("\u{25CB}", MUTED)
        };
        Line::from(vec![
            Span::styled(format!(" {sym} "), Style::default().fg(color)),
            Span::styled(format!("{name:<12}"), Style::default().fg(TEXT)),
            Span::styled(
                if running { "running" } else { "stopped" },
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {extra}"), Style::default().fg(MUTED)),
        ])
    };

    let dolt_extra = app
        .infra
        .dolt_port
        .map(|p| format!("port {p}"))
        .unwrap_or_default();

    let lines = vec![
        status_line("Daemon", app.infra.daemon_running, ""),
        status_line("Dolt", app.infra.dolt_running, &dolt_extra),
        status_line(
            "Tmux",
            app.infra.tmux_running,
            &format!("{} sessions", app.infra.tmux_sessions),
        ),
        Line::default(),
        Line::from(vec![
            Span::styled("   Overseer: ", Style::default().fg(MUTED)),
            Span::styled(&app.overseer_name, Style::default().fg(TEXT_BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("   Workspace: ", Style::default().fg(MUTED)),
            Span::styled(
                app.infra
                    .workspace
                    .as_deref()
                    .unwrap_or("not found"),
                Style::default().fg(ACCENT_DIM),
            ),
        ]),
        Line::default(),
        Line::from(vec![
            Span::styled("   Agents: ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{} running", app.agents_running()),
                Style::default().fg(if app.agents_running() > 0 {
                    GREEN
                } else {
                    MUTED
                }),
            ),
            Span::styled(
                format!("  {} with work", app.agents_with_work()),
                Style::default().fg(if app.agents_with_work() > 0 {
                    CYAN
                } else {
                    MUTED
                }),
            ),
        ]),
    ];

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}

fn draw_recent_beads(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Recent Beads (user tasks) ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.beads.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No beads found. Create tasks with 's' to spawn.",
                Style::default().fg(MUTED),
            )),
            inner,
        );
        return;
    }

    let items: Vec<ListItem> = app
        .beads
        .iter()
        .take(inner.height as usize)
        .map(|bead| {
            let (status_sym, status_color) = match bead.status.as_str() {
                "open" => ("\u{25CB}", MUTED),
                "in_progress" => ("\u{25C9}", CYAN),
                "closed" => ("Y", GREEN),
                "blocked" => ("X", RED),
                _ => ("?", MUTED),
            };
            let priority_color = match bead.priority {
                0 => RED,
                1 => ORANGE,
                2 => YELLOW,
                3 => MUTED,
                _ => Color::Rgb(50, 50, 60),
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{status_sym} "),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("P{} ", bead.priority),
                    Style::default().fg(priority_color),
                ),
                Span::styled(
                    format!("{:<10} ", bead.id),
                    Style::default().fg(ACCENT_DIM),
                ),
                Span::styled(
                    format!("{:<12} ", bead.rig),
                    Style::default().fg(PURPLE),
                ),
                Span::styled(&bead.title, Style::default().fg(TEXT)),
                if let Some(ref assignee) = bead.assignee {
                    Span::styled(format!("  -> {assignee}"), Style::default().fg(CYAN))
                } else {
                    Span::raw("")
                },
            ]))
        })
        .collect();

    frame.render_widget(List::new(items), inner);
}

// ── Agents tab ───────────────────────────────────────────────────────

fn draw_agents(frame: &mut Frame, app: &mut App, area: Rect) {
    let count = app.filtered_agents().len();
    app.max_scroll = count.saturating_sub(1);
    let agents = app.filtered_agents();

    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Address").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Role").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("State").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Run").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Work").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Runtime").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Hook Bead").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Mail").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = agents
        .iter()
        .skip(app.scroll)
        .map(|agent| {
            let running_color = if agent.running { GREEN } else { MUTED };
            let work_color = if agent.has_work { CYAN } else { MUTED };
            let role = agent.role.as_deref().unwrap_or("\u{2014}");
            let state = agent.state.as_deref().unwrap_or("\u{2014}");
            let state_color = match state {
                "active" | "working" => GREEN,
                "idle" => MUTED,
                "stuck" => YELLOW,
                _ => TEXT,
            };
            let runtime = agent.runtime.as_deref().unwrap_or("\u{2014}");
            let hook = agent.hook_bead.as_deref().unwrap_or("\u{2014}");
            let mail = agent.unread_mail;

            Row::new(vec![
                Cell::from(agent.name.clone()).style(Style::default().fg(TEXT_BRIGHT)),
                Cell::from(agent.address.clone()).style(Style::default().fg(ACCENT_DIM)),
                Cell::from(role.to_string()).style(Style::default().fg(PURPLE)),
                Cell::from(state.to_string()).style(
                    Style::default()
                        .fg(state_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(if agent.running { "Y" } else { "N" })
                    .style(Style::default().fg(running_color)),
                Cell::from(if agent.has_work { "Y" } else { "\u{2014}" })
                    .style(Style::default().fg(work_color)),
                Cell::from(runtime.to_string()).style(Style::default().fg(MUTED)),
                Cell::from(hook.to_string()).style(
                    Style::default().fg(if hook != "\u{2014}" { CYAN } else { MUTED }),
                ),
                Cell::from(if mail > 0 {
                    format!("{mail}")
                } else {
                    "\u{2014}".into()
                })
                .style(
                    Style::default().fg(if mail > 0 { YELLOW } else { MUTED }),
                ),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(22),
        Constraint::Length(14),
        Constraint::Length(8),
        Constraint::Length(5),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths)
        .header(header.style(Style::default().bg(SURFACE)))
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Agents ({count}) "),
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
    let mut sb_state = ScrollbarState::new(count).position(app.scroll);
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
            format!(" Convoys ({count}) "),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(SURFACE))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if convoys.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No convoys. Create one with: gt convoy create \"name\" <bead-id>",
                Style::default().fg(MUTED),
            )),
            inner,
        );
        return;
    }

    let constraints: Vec<Constraint> = convoys
        .iter()
        .skip(app.scroll)
        .take(inner.height as usize / 4)
        .map(|_| Constraint::Length(4))
        .collect();
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

        let status_color = match convoy.status.as_str() {
            "open" => GREEN,
            "closed" => CYAN,
            "staged_ready" => YELLOW,
            "staged_warnings" => ORANGE,
            _ => MUTED,
        };

        let progress = convoy.progress;
        let pct = (progress * 100.0) as u16;

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", convoy.id), Style::default().fg(ACCENT_DIM)),
                Span::styled(
                    &convoy.title,
                    Style::default()
                        .fg(TEXT_BRIGHT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    &convoy.status,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
            line1,
        );

        let gauge_color = match progress {
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
                format!("  {}/{} ({pct}%)", convoy.completed, convoy.total),
                Style::default().fg(TEXT),
            ))
            .ratio(progress);
        frame.render_widget(gauge, line2);

        if !convoy.workers.is_empty() {
            let worker_spans: Vec<Span> = convoy
                .workers
                .iter()
                .map(|w| {
                    let color = if w.blocked { RED } else { CYAN };
                    Span::styled(format!("{}  ", w.name), Style::default().fg(color))
                })
                .collect();
            let mut spans = vec![Span::styled("  Workers: ", Style::default().fg(MUTED))];
            spans.extend(worker_spans);
            frame.render_widget(Paragraph::new(Line::from(spans)), line3);
        }
    }
}

// ── Beads tab ────────────────────────────────────────────────────────

fn draw_beads(frame: &mut Frame, app: &mut App, area: Rect) {
    let count = app.filtered_beads().len();
    app.max_scroll = count.saturating_sub(1);
    let beads = app.filtered_beads();

    let header = Row::new(vec![
        Cell::from("ID").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("P").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Title").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Rig").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Assignee").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = beads
        .iter()
        .skip(app.scroll)
        .map(|bead| {
            let status_color = match bead.status.as_str() {
                "open" => MUTED,
                "in_progress" => CYAN,
                "closed" => GREEN,
                "blocked" => RED,
                _ => MUTED,
            };
            let priority_color = match bead.priority {
                0 => RED,
                1 => ORANGE,
                2 => YELLOW,
                3 => MUTED,
                _ => Color::Rgb(50, 50, 60),
            };
            Row::new(vec![
                Cell::from(bead.id.clone()).style(Style::default().fg(ACCENT_DIM)),
                Cell::from(format!("P{}", bead.priority))
                    .style(Style::default().fg(priority_color)),
                Cell::from(bead.status.clone()).style(
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(bead.issue_type.clone()).style(Style::default().fg(MUTED)),
                Cell::from(bead.title.clone()).style(Style::default().fg(TEXT)),
                Cell::from(bead.rig.clone()).style(Style::default().fg(PURPLE)),
                Cell::from(
                    bead.assignee
                        .clone()
                        .unwrap_or_else(|| "\u{2014}".into()),
                )
                .style(Style::default().fg(CYAN)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(4),
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Min(30),
        Constraint::Length(14),
        Constraint::Length(14),
    ];

    let table = Table::new(rows, widths)
        .header(header.style(Style::default().bg(SURFACE)))
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Beads ({count}) "),
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
    let mut sb_state = ScrollbarState::new(count).position(app.scroll);
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
        Cell::from("Sync").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
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
                format!("+{} -{}", repo.ahead, repo.behind)
            } else if repo.ahead > 0 {
                format!("+{}", repo.ahead)
            } else if repo.behind > 0 {
                format!("-{}", repo.behind)
            } else {
                "synced".into()
            };
            let sync_color = if repo.ahead > 0 || repo.behind > 0 {
                ORANGE
            } else {
                MUTED
            };
            let commit = if repo.last_commit.len() > 40 {
                format!("{}...", &repo.last_commit[..39])
            } else {
                repo.last_commit.clone()
            };

            Row::new(vec![
                Cell::from(repo.name.clone())
                    .style(Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD)),
                Cell::from(repo.branch.clone()).style(Style::default().fg(PURPLE)),
                Cell::from(status_text).style(
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(sync).style(Style::default().fg(sync_color)),
                Cell::from(commit).style(Style::default().fg(MUTED)),
                Cell::from(repo.path.clone()).style(Style::default().fg(ACCENT_DIM)),
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
                    format!(" Repositories ({count}) "),
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
    let mut sb_state = ScrollbarState::new(count).position(app.scroll);
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
    let dialog_width = 55u16.min(area.width.saturating_sub(4));
    let dialog_height = 16u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(Span::styled(
            " Spawn Agent (gt sling) ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(SURFACE));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let [rig_hint_area, field1, field2, field3, _gap, help_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .areas(inner);

    let rig_names: Vec<String> = app.rigs.iter().map(|r| r.name.clone()).collect();
    let rig_hint = Paragraph::new(Line::from(vec![
        Span::styled(" Rigs: ", Style::default().fg(MUTED)),
        Span::styled(rig_names.join(", "), Style::default().fg(ACCENT_DIM)),
    ]));
    frame.render_widget(rig_hint, rig_hint_area);

    let field_style = |active: bool| {
        if active {
            Style::default().fg(TEXT_BRIGHT)
        } else {
            Style::default().fg(MUTED)
        }
    };
    let label_style = |active: bool| {
        if active {
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED)
        }
    };

    let rig_text = if app.spawn_rig.is_empty() && app.spawn_field == 0 {
        "type rig name..."
    } else if app.spawn_rig.is_empty() {
        ""
    } else {
        &app.spawn_rig
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Rig:     ", label_style(app.spawn_field == 0)),
            Span::styled(rig_text, field_style(app.spawn_field == 0)),
            if app.spawn_field == 0 {
                Span::styled("\u{2588}", Style::default().fg(ACCENT))
            } else {
                Span::raw("")
            },
        ])),
        field1,
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Runtime: ", label_style(app.spawn_field == 1)),
            Span::styled(
                format!("< {} >", app.spawn_runtime()),
                field_style(app.spawn_field == 1),
            ),
        ])),
        field2,
    );

    let task_text = if app.spawn_task.is_empty() && app.spawn_field == 2 {
        "describe the task..."
    } else if app.spawn_task.is_empty() {
        ""
    } else {
        &app.spawn_task
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Task:    ", label_style(app.spawn_field == 2)),
            Span::styled(task_text, field_style(app.spawn_field == 2)),
            if app.spawn_field == 2 {
                Span::styled("\u{2588}", Style::default().fg(ACCENT))
            } else {
                Span::raw("")
            },
        ])),
        field3,
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Tab", Style::default().fg(ACCENT)),
            Span::styled(":next  ", Style::default().fg(MUTED)),
            Span::styled("Up/Dn", Style::default().fg(ACCENT)),
            Span::styled(":runtime  ", Style::default().fg(MUTED)),
            Span::styled("Enter", Style::default().fg(ACCENT)),
            Span::styled(":spawn  ", Style::default().fg(MUTED)),
            Span::styled("Esc", Style::default().fg(ACCENT)),
            Span::styled(":cancel", Style::default().fg(MUTED)),
        ])),
        help_area,
    );
}
