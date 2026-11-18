use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::{
    app::{App, Screen},
    categories, theme,
};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    let popup_area = centered_rect(80, 60, size);

    f.render_widget(Clear, popup_area);

    let enter_key = match app.screen {
        Screen::Category(categories::WORKSPACE_INDEX) => "help.workspace",
        Screen::Action => "help.run",
        _ => "help.open",
    };
    let mut lines = vec![
        Line::styled(app.text("help.shortcuts"), Style::default().fg(theme::HELP_TITLE)),
        Line::from(""),
    ];
    let shortcuts = [
        "help.navigate",
        enter_key,
        "help.capture",
        "workspace.guide",
        "help.zero",
        "help.back",
        "help.toggle",
        "help.quit",
    ];
    lines.extend(shortcuts.map(|key| {
        Line::styled(app.text(key), Style::default().fg(theme::HELP_TEXT))
    }));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::HELP_BORDER))
        .title(format!(" {} ", app.text("help.title")));

    let paragraph = Paragraph::new(lines).block(block);

    f.render_widget(paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
