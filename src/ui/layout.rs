use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Split the terminal into: header (3), main content (variable), sparkline (5), footer (1)
pub fn main_layout(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header with tabs + stats
            Constraint::Min(10),   // main content area
            Constraint::Length(5), // sparkline area
            Constraint::Length(1), // footer keybindings
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2], chunks[3])
}
