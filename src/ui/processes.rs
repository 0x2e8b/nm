use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;
use crate::data::model::SortField;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let header_cells = [
        ("Process", SortField::Name),
        ("PID", SortField::Pid),
        ("Conn", SortField::Connections),
        ("Down", SortField::BytesIn),
        ("Up", SortField::BytesOut),
        ("Rate In", SortField::RateIn),
        ("Rate Out", SortField::RateOut),
    ]
    .iter()
    .map(|(label, field)| {
        let text = if app.sort_field == *field {
            format!("{} ▼", label)
        } else {
            label.to_string()
        };
        Cell::from(Span::styled(text, theme::header_style()))
    })
    .collect::<Vec<_>>();

    let header = Row::new(header_cells).height(1);

    let max_rate = app
        .snapshot
        .processes
        .iter()
        .map(|p| p.rate_in.max(p.rate_out))
        .fold(0.0_f64, f64::max);

    let rows: Vec<Row> = app
        .filtered_processes()
        .iter()
        .map(|p| {
            let rate_color = theme::rate_color(p.rate_in.max(p.rate_out));
            let bar = theme::rate_bar(p.rate_in + p.rate_out, max_rate * 2.0);
            Row::new(vec![
                Cell::from(p.name.clone()),
                Cell::from(p.pid.to_string()),
                Cell::from(p.connection_count().to_string()),
                Cell::from(format_bytes(p.bytes_in)),
                Cell::from(format_bytes(p.bytes_out)),
                Cell::from(Span::styled(
                    format_rate(p.rate_in),
                    Style::default().fg(theme::rate_color(p.rate_in)),
                )),
                Cell::from(Span::styled(
                    format!("{} {}", format_rate(p.rate_out), bar),
                    Style::default().fg(rate_color),
                )),
            ])
        })
        .collect();

    let widths = [
        Constraint::Min(16),
        Constraint::Length(7),
        Constraint::Length(5),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(18),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR))
                .title(" Processes "),
        )
        .row_highlight_style(theme::selected_style())
        .highlight_symbol("▸ ");

    let mut state = TableState::default();
    state.select(Some(app.process_index));
    f.render_stateful_widget(table, area, &mut state);
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_048_576.0)
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else if bytes_per_sec > 0.0 {
        format!("{:.0} B/s", bytes_per_sec)
    } else {
        "—".to_string()
    }
}
