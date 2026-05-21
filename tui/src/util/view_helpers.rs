use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::Widget;

/// Draws the bottom menu
pub fn view_bottom_menu(lines: &[&str], area: Rect, buf: &mut Buffer) {
    let text = Text::from_iter(lines.iter().map(|l| Line::raw(*l)))
        .fg(Color::DarkGray)
        .centered();

    let mut menu_area = area.centered_horizontally(Constraint::Length(text.width() as u16));
    menu_area.y = area.bottom().saturating_sub(text.height() as u16);

    text.render(menu_area, buf);
}

/// To compute the blinking cursor
pub fn should_draw_cursor() -> bool {
    let t = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => 0,
    };

    t % 2 == 0
}
