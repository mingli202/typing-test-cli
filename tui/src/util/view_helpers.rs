use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::Widget;

pub fn view_bottom_menu(lines: &[&str], area: Rect, buf: &mut Buffer) {
    let text = Text::from_iter(lines.iter().map(|l| Line::raw(*l)))
        .fg(Color::DarkGray)
        .centered();

    let mut menu_area = area.centered_horizontally(Constraint::Length(text.width() as u16));
    menu_area.y = area.bottom().saturating_sub(text.height() as u16);

    text.render(menu_area, buf);
}
