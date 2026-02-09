use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;
use crate::ui::processes::{format_bytes, format_rate};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let header_cells = ["Process", "Protocol", "Local", "Remote", "State", "Down", "Up"]
        .iter()
        .map(|h| Cell::from(Span::styled(*h, theme::header_style())))
        .collect::<Vec<_>>();

    let header = Row::new(header_cells).height(1);

    let mut rows: Vec<Row> = Vec::new();

    let processes = app.filtered_processes();
    for p in &processes {
        for conn in &p.connections {
            let remote_display = conn
                .hostname
                .as_deref()
                .unwrap_or(&conn.remote_addr);

            let remote_str = if conn.remote_port > 0 {
                format!("{}:{}", remote_display, conn.remote_port)
            } else {
                remote_display.to_string()
            };

            let local_str = if conn.local_port > 0 {
                format!("{}:{}", conn.local_addr, conn.local_port)
            } else {
                conn.local_addr.clone()
            };

            // Apply filter
            if let Some(ref filter) = app.filter_text {
                let filter_lower = filter.to_lowercase();
                let matches = p.name.to_lowercase().contains(&filter_lower)
                    || remote_str.to_lowercase().contains(&filter_lower)
                    || local_str.to_lowercase().contains(&filter_lower)
                    || conn.protocol.to_string().to_lowercase().contains(&filter_lower);
                if !matches {
                    continue;
                }
            }

            rows.push(Row::new(vec![
                Cell::from(p.name.clone()),
                Cell::from(conn.protocol.to_string()),
                Cell::from(local_str),
                Cell::from(remote_str),
                Cell::from(conn.state.clone()),
                Cell::from(format_bytes(conn.bytes_in)),
                Cell::from(format_rate(conn.bytes_out as f64)),
            ]));
        }
    }

    let widths = [
        Constraint::Min(14),
        Constraint::Length(5),
        Constraint::Length(22),
        Constraint::Min(28),
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR))
                .title(" Connections "),
        )
        .row_highlight_style(theme::selected_style())
        .highlight_symbol("▸ ");

    let mut state = TableState::default();
    state.select(Some(app.connection_index));
    f.render_stateful_widget(table, area, &mut state);
}
