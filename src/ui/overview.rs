use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
use ratatui::Frame;

use crate::app::App;
use crate::ui::processes::{format_bytes, format_rate};
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Stats summary
            Constraint::Min(8),   // Top processes
            Constraint::Length(5), // Sparkline
        ])
        .split(area);

    // Stats summary
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER_COLOR))
        .title(" Overview ");

    let stats_text = vec![
        Line::from(vec![
            Span::styled("Total Down: ", theme::header_style()),
            Span::styled(
                format_bytes(app.snapshot.total_bytes_in),
                Style::default().fg(theme::DOWNLOAD_COLOR),
            ),
            Span::raw("  "),
            Span::styled("Total Up: ", theme::header_style()),
            Span::styled(
                format_bytes(app.snapshot.total_bytes_out),
                Style::default().fg(theme::UPLOAD_COLOR),
            ),
        ]),
        Line::from(vec![
            Span::styled("Rate In: ", theme::header_style()),
            Span::styled(
                format_rate(app.snapshot.total_rate_in),
                Style::default().fg(theme::rate_color(app.snapshot.total_rate_in)),
            ),
            Span::raw("  "),
            Span::styled("Rate Out: ", theme::header_style()),
            Span::styled(
                format_rate(app.snapshot.total_rate_out),
                Style::default().fg(theme::rate_color(app.snapshot.total_rate_out)),
            ),
            Span::raw("  "),
            Span::styled("Connections: ", theme::header_style()),
            Span::raw(app.snapshot.total_connections.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Processes: ", theme::header_style()),
            Span::raw(app.snapshot.processes.len().to_string()),
        ]),
    ];

    let stats = Paragraph::new(stats_text).block(stats_block);
    f.render_widget(stats, chunks[0]);

    // Top processes by rate
    let top_procs: Vec<Line> = app
        .snapshot
        .processes
        .iter()
        .take(10)
        .map(|p| {
            Line::from(vec![
                Span::styled(
                    format!("{:<20}", p.name),
                    Style::default().fg(theme::ACTIVE_TAB_FG),
                ),
                Span::styled(
                    format!("▼{} ", format_rate(p.rate_in)),
                    Style::default().fg(theme::rate_color(p.rate_in)),
                ),
                Span::styled(
                    format!("▲{}", format_rate(p.rate_out)),
                    Style::default().fg(theme::rate_color(p.rate_out)),
                ),
            ])
        })
        .collect();

    let top_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER_COLOR))
        .title(" Top Processes ");
    let top = Paragraph::new(top_procs).block(top_block);
    f.render_widget(top, chunks[1]);

    // Bandwidth sparkline
    render_sparkline(f, chunks[2], app);
}

pub fn render_sparkline(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER_COLOR))
        .title(" Bandwidth ");

    let data: Vec<u64> = app
        .bandwidth_history
        .iter()
        .map(|&v| v as u64)
        .collect();

    let sparkline = Sparkline::default()
        .block(block)
        .data(&data)
        .style(Style::default().fg(theme::DOWNLOAD_COLOR));

    f.render_widget(sparkline, area);
}

/// Render the mini sparkline shown in the footer area
pub fn render_footer_sparkline(f: &mut Frame, area: Rect, app: &App) {
    let data: Vec<u64> = app
        .bandwidth_history
        .iter()
        .map(|&v| v as u64)
        .collect();

    let sparkline = Sparkline::default()
        .data(&data)
        .style(Style::default().fg(theme::DOWNLOAD_COLOR));

    f.render_widget(sparkline, area);
}
