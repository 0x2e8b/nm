mod app;
mod config;
mod data;
mod ui;

use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Terminal;

use app::{ActiveTab, App};
use config::Config;
use ui::theme;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();
    let sort_field = config.parse_sort_field();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(sort_field, config.interval);

    // Initial data fetch
    app.update_data().await;

    let tick_rate = Duration::from_secs(config.interval);

    loop {
        // Draw
        terminal.draw(|f| draw_ui(f, &app))?;

        // Handle events with timeout
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if app.filtering {
                    match key.code {
                        KeyCode::Enter => app.apply_filter(),
                        KeyCode::Esc => app.cancel_filter(),
                        KeyCode::Backspace => { app.filter_input.pop(); }
                        KeyCode::Char(c) => app.filter_input.push(c),
                        _ => {}
                    }
                } else if app.show_help {
                    match key.code {
                        KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
                            app.show_help = false;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.should_quit = true;
                        }
                        KeyCode::Tab => app.active_tab = app.active_tab.next(),
                        KeyCode::BackTab => app.active_tab = app.active_tab.prev(),
                        KeyCode::Char('j') | KeyCode::Down => app.nav_down(),
                        KeyCode::Char('k') | KeyCode::Up => app.nav_up(),
                        KeyCode::Char('s') => app.cycle_sort(),
                        KeyCode::Char('/') => app.enter_filter(),
                        KeyCode::Esc => app.cancel_filter(),
                        KeyCode::Char('p') => app.paused = !app.paused,
                        KeyCode::Char('?') => app.show_help = true,
                        KeyCode::Enter => app.drill_down(),
                        _ => {}
                    }
                }
            }
        } else {
            // Tick — refresh data
            app.update_data().await;
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn draw_ui(f: &mut ratatui::Frame, app: &App) {
    let (header_area, main_area, sparkline_area, footer_area) =
        ui::layout::main_layout(f.area());

    // Header: tabs + stats
    draw_header(f, header_area, app);

    // Main content based on active tab
    match app.active_tab {
        ActiveTab::Processes => ui::processes::render(f, main_area, app),
        ActiveTab::Connections => ui::connections::render(f, main_area, app),
        ActiveTab::Overview => ui::overview::render(f, main_area, app),
    }

    // Sparkline
    ui::overview::render_footer_sparkline(f, sparkline_area, app);

    // Footer
    draw_footer(f, footer_area, app);

    // Help overlay
    if app.show_help {
        ui::help::render(f);
    }
}

fn draw_header(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(40)])
        .split(area);

    // Tabs
    let tab_titles = vec![
        Span::raw(" Processes "),
        Span::raw(" Connections "),
        Span::raw(" Overview "),
    ];
    let selected = match app.active_tab {
        ActiveTab::Processes => 0,
        ActiveTab::Connections => 1,
        ActiveTab::Overview => 2,
    };
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).border_style(
            Style::default().fg(theme::BORDER_COLOR),
        ))
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(theme::ACTIVE_TAB_FG)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(theme::INACTIVE_TAB_FG));

    f.render_widget(tabs, chunks[0]);

    // Stats summary
    let stats = format!(
        "▼ {} ▲ {} │ {} conn",
        ui::processes::format_rate(app.snapshot.total_rate_in),
        ui::processes::format_rate(app.snapshot.total_rate_out),
        app.snapshot.total_connections,
    );
    let paused = if app.paused { " [PAUSED]" } else { "" };
    let stats_widget = Paragraph::new(format!("{}{}", stats, paused))
        .block(Block::default().borders(Borders::ALL).border_style(
            Style::default().fg(theme::BORDER_COLOR),
        ))
        .style(theme::header_style());
    f.render_widget(stats_widget, chunks[1]);
}

fn draw_footer(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
    let text = if app.filtering {
        format!("Filter: {}█", app.filter_input)
    } else if let Some(ref filter) = app.filter_text {
        format!(
            "Tab: switch │ j/k: nav │ s: sort ({}) │ /: filter [{}] │ ?: help │ q: quit",
            app.sort_field.label(),
            filter
        )
    } else {
        format!(
            "Tab: switch │ j/k: nav │ s: sort ({}) │ /: filter │ Enter: drill │ p: pause │ ?: help │ q: quit",
            app.sort_field.label()
        )
    };

    let footer = Paragraph::new(text).style(theme::footer_style());
    f.render_widget(footer, area);
}
