use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
    widgets::{ListItem, Padding, Paragraph, Wrap},
};

use crate::actions;

use super::{app::App, categories, header, theme, views};

pub fn render(f: &mut Frame, app: &App) {
    let [head, content, footer] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .areas(f.area());
    header::render_header(f, head, app);

    let list_area = if content.width >= 88 {
        let [left, _, right] = Layout::horizontal([
            Constraint::Fill(2),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(content);
        overview(f, right, app);
        left
    } else {
        content
    };
    let items = categories::LIST.iter().enumerate().map(|(index, category)| {
        ListItem::new(format!("[{}] {}  {}", categories::number(index), category.icon, app.text(category.name_key)))
    }).collect();
    views::list(f, list_area, app.text("menu.title").into(), items, app.selected);
    views::status(f, footer, app, "menu.hint");
}

fn overview(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let selected = &categories::LIST[app.selected];
    let count = actions::in_category(categories::number(app.selected)).count();
    let caption = match app.selected {
        categories::WORKSPACE_INDEX => app.text("menu.workspace_caption").to_owned(),
        21 => app.text("menu.tutor_caption").to_owned(),
        _ => format!("{count} {}", app.text("menu.actions_available")),
    };
    let lines = [
        Line::styled(app.text(selected.name_key), Style::default().fg(theme::ACCENT)),
        Line::raw(caption),
        Line::raw(""),
        Line::raw(app.text("menu.capture_shortcut")),
        Line::raw(app.text("menu.log_shortcut")),
        Line::raw(app.text("workspace.guide")),
    ];
    f.render_widget(
        Paragraph::new(lines.to_vec()).wrap(Wrap { trim: true })
            .block(views::block(app.text("menu.overview")).padding(Padding::horizontal(1))),
        area,
    );
}
