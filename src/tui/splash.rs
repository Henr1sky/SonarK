use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{app::App, theme};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(20),
            Constraint::Length(4),
            Constraint::Percentage(30),
        ])
        .split(size);

    let banner_text = Text::from(crate::BANNER);
    let banner_paragraph = Paragraph::new(banner_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::SPLASH_BORDER))
                .title(format!(" SonarK v{} ", app.version)),
        )
        .alignment(Alignment::Center);
    f.render_widget(banner_paragraph, outer[1]);

    let hint = Paragraph::new(Line::from(vec![Span::styled(
        app.text("splash.hint"),
        Style::default().fg(theme::SPLASH_HINT),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(hint, outer[2]);
}
