use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::theme;

pub fn render(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());

    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled(
            " Network Monitor — Help ",
            Style::default()
                .fg(theme::HEADER_FG)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab / Shift-Tab  ", Style::default().fg(theme::ACTIVE_TAB_FG)),
            Span::raw("Switch between tabs"),
        ]),
        Line::from(vec![
            Span::styled("j / k / ↑ / ↓    ", Style::default().fg(theme::ACTIVE_TAB_FG)),
            Span::raw("Navigate rows"),
        ]),
        Line::from(vec![
            Span::styled("Enter            ", Style::default().fg(theme::ACTIVE_TAB_FG)),
            Span::raw("Drill into process connections"),
        ]),
        Line::from(vec![
            Span::styled("s                ", Style::default().fg(theme::ACTIVE_TAB_FG)),
            Span::raw("Cycle sort field"),
        ]),
        Line::from(vec![
            Span::styled("/                ", Style::default().fg(theme::ACTIVE_TAB_FG)),
            Span::raw("Filter processes/connections"),
        ]),
        Line::from(vec![
            Span::styled("Esc              ", Style::default().fg(theme::ACTIVE_TAB_FG)),
            Span::raw("Clear filter / close help"),
        ]),
        Line::from(vec![
            Span::styled("p                ", Style::default().fg(theme::ACTIVE_TAB_FG)),
            Span::raw("Pause/resume data collection"),
        ]),
        Line::from(vec![
            Span::styled("?                ", Style::default().fg(theme::ACTIVE_TAB_FG)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("q                ", Style::default().fg(theme::ACTIVE_TAB_FG)),
            Span::raw("Quit"),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::HEADER_FG))
        .title(" Help ");

    let help = Paragraph::new(help_text).block(block);
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
