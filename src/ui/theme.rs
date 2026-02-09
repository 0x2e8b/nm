use ratatui::style::{Color, Modifier, Style};

pub const HEADER_FG: Color = Color::Cyan;
pub const ACTIVE_TAB_FG: Color = Color::White;
pub const ACTIVE_TAB_BG: Color = Color::DarkGray;
pub const INACTIVE_TAB_FG: Color = Color::Gray;
pub const SELECTED_BG: Color = Color::DarkGray;
pub const BORDER_COLOR: Color = Color::DarkGray;
pub const FOOTER_FG: Color = Color::DarkGray;
pub const UPLOAD_COLOR: Color = Color::Magenta;
pub const DOWNLOAD_COLOR: Color = Color::Blue;

pub fn rate_color(bytes_per_sec: f64) -> Color {
    if bytes_per_sec > 1_000_000.0 {
        Color::Red
    } else if bytes_per_sec > 100_000.0 {
        Color::Yellow
    } else if bytes_per_sec > 0.0 {
        Color::Green
    } else {
        Color::DarkGray
    }
}

pub fn header_style() -> Style {
    Style::default().fg(HEADER_FG).add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().bg(SELECTED_BG).add_modifier(Modifier::BOLD)
}

pub fn normal_style() -> Style {
    Style::default()
}

pub fn footer_style() -> Style {
    Style::default().fg(FOOTER_FG)
}

/// Returns a bar string representing the rate visually
pub fn rate_bar(rate: f64, max_rate: f64) -> String {
    let chars = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    let width = 6;
    if max_rate <= 0.0 || rate <= 0.0 {
        return " ".repeat(width);
    }
    let ratio = (rate / max_rate).min(1.0);
    let filled = ratio * width as f64;
    let full_blocks = filled as usize;
    let partial = ((filled - full_blocks as f64) * 8.0) as usize;

    let mut bar = String::new();
    for _ in 0..full_blocks {
        bar.push('█');
    }
    if full_blocks < width && partial > 0 {
        bar.push(chars[partial.min(7)]);
    }
    while bar.chars().count() < width {
        bar.push(' ');
    }
    bar
}
