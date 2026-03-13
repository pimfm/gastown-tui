mod api_client;
mod app;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    enable_raw_mode()?;
    let mut stdout = io::stderr();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stderr>>) -> Result<()> {
    let mut app = App::new();
    app.refresh_all();

    let tick_rate = Duration::from_millis(250);
    let refresh_rate = Duration::from_secs(5);
    let mut last_tick = Instant::now();
    let mut last_refresh = Instant::now();

    loop {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.is_filtering() {
                        match key.code {
                            KeyCode::Esc => app.dismiss(),
                            KeyCode::Enter => app.toggle_filter(),
                            KeyCode::Backspace => app.filter_pop(),
                            KeyCode::Char(c) => app.filter_push(c),
                            _ => {}
                        }
                    } else if app.show_spawn_dialog {
                        match key.code {
                            KeyCode::Esc => app.dismiss(),
                            KeyCode::Enter => app.confirm_spawn(),
                            KeyCode::Tab => app.spawn_next_field(),
                            KeyCode::Backspace => app.spawn_backspace(),
                            KeyCode::Char(c) => app.spawn_type_char(c),
                            KeyCode::Up => app.spawn_prev_option(),
                            KeyCode::Down => app.spawn_next_option(),
                            _ => {}
                        }
                    } else {
                        match (key.modifiers, key.code) {
                            (_, KeyCode::Char('q'))
                            | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                                return Ok(());
                            }
                            (_, KeyCode::Tab) => app.next_tab(),
                            (KeyModifiers::SHIFT, KeyCode::BackTab) => app.prev_tab(),
                            (_, KeyCode::Char('1')) => app.select_tab(0),
                            (_, KeyCode::Char('2')) => app.select_tab(1),
                            (_, KeyCode::Char('3')) => app.select_tab(2),
                            (_, KeyCode::Char('4')) => app.select_tab(3),
                            (_, KeyCode::Char('5')) => app.select_tab(4),
                            (_, KeyCode::Char('r')) => app.refresh_all(),
                            (_, KeyCode::Char('j')) | (_, KeyCode::Down) => app.scroll_down(),
                            (_, KeyCode::Char('k')) | (_, KeyCode::Up) => app.scroll_up(),
                            (_, KeyCode::Char('g')) | (_, KeyCode::Home) => app.scroll_top(),
                            (_, KeyCode::Char('G')) | (_, KeyCode::End) => app.scroll_bottom(),
                            (_, KeyCode::Enter) => app.action_enter(),
                            (_, KeyCode::Char('s')) => app.action_spawn(),
                            (_, KeyCode::Char('/')) => app.toggle_filter(),
                            (_, KeyCode::Esc) => app.dismiss(),
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if last_refresh.elapsed() >= refresh_rate {
            app.refresh_all();
            last_refresh = Instant::now();
        }
    }
}
