use ratatui::style::{Color, Modifier, Style};

pub const ACCENT: Color = Color::LightCyan;
pub const MUTED: Color = Color::Gray;
pub const NOTICE: Color = Color::LightYellow;
pub const FIELD_BACKGROUND: Color = Color::Rgb(32, 43, 53);

pub const HEADER_BORDER: Color = Color::LightCyan;
pub const HEADER_TEXT: Color = Color::White;
pub const HEADER_LABEL: Color = Color::Gray;
pub const HEADER_VALUE: Color = Color::Yellow;

pub const MENU_BORDER: Color = Color::Cyan;
pub const MENU_ITEM: Color = Color::White;
pub const MENU_SELECTED: Color = Color::Black;
pub const MENU_SELECTED_BG: Color = Color::LightCyan;
pub const MENU_HINT: Color = Color::DarkGray;

pub const CATEGORY_BORDER: Color = Color::Rgb(67, 86, 103);
pub const CATEGORY_TITLE: Color = Color::Yellow;
pub const CATEGORY_BODY: Color = Color::Gray;

pub const SPLASH_BORDER: Color = Color::Cyan;
pub const SPLASH_TEXT: Color = Color::Yellow;
pub const SPLASH_HINT: Color = Color::DarkGray;

pub const HELP_BORDER: Color = Color::Magenta;
pub const HELP_TITLE: Color = Color::Yellow;
pub const HELP_TEXT: Color = Color::White;

pub fn selected_style() -> Style {
    Style::default()
        .fg(MENU_SELECTED)
        .bg(MENU_SELECTED_BG)
        .add_modifier(Modifier::BOLD)
}