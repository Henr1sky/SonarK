use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::{app::App, theme};

pub fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let capture = app.project.data.input_path()
        .map_or("-".into(), |path| path.to_string_lossy());
    let capture = crate::tshark::sanitize(&capture);
    let marker = if app.project.data.active_derived().is_some() {
        app.text("header.derived")
    } else {
        ""
    };
    let label = Style::default().fg(theme::HEADER_LABEL);
    let value = Style::default().fg(theme::HEADER_VALUE);
    let lines = [
        Line::from(vec![
            Span::styled(format!("v{}   ", app.version), value),
            Span::styled(app.text("header.project"), label),
            Span::styled(&app.project.name, value),
        ]),
        Line::from(vec![
            Span::styled(app.text("header.capture"), label),
            Span::styled(marker, value),
            Span::styled(capture, value),
        ]),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::HEADER_BORDER))
        .title(" SonarK ");
    f.render_widget(Paragraph::new(lines.to_vec()).block(block), area);
}
