use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::{ListItem, Padding, Paragraph, Wrap},
};

use super::{app::App, categories, header, views};

pub fn render(f: &mut Frame, app: &App, index: usize) {
    let category = &categories::LIST[index];
    let title = format!(" [{}] {} ", categories::number(index), app.text(category.name_key));
    if index == categories::WORKSPACE_INDEX {
        workspace(f, app, title);
    } else {
        actions(f, app, title);
    }
}

pub fn favorites(f: &mut Frame, app: &App) {
    actions(f, app, app.text("favorite.title").to_owned());
}

fn category_layout(f: &Frame) -> [ratatui::layout::Rect; 3] {
    Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(2),
        Constraint::Length(3),
    ])
    .areas(f.area())
}

fn workspace(f: &mut Frame, app: &App, title: String) {
    let [header_area, body, footer] = category_layout(f);
    header::render_header(f, header_area, app);
    let language = app.language();
    let [sources, project, settings] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(8),
        Constraint::Min(3),
    ]).areas(body);
    let inputs = format!("{}\n{}\n{}", app.text("workspace.capture"),
        app.text("workspace.log"), if app.project.data.derived_capture.is_some() {
            app.text("workspace.derived")
        } else {
            app.text("workspace.guide")
        });
    let language_text = format!("{}{}\n{}{}", app.text("workspace.language"),
        language.name(), app.text("workspace.change"), language.other().name());
    let mut settings_text = language_text;
    if let Some((key, error)) = &app.config_error {
        settings_text.push_str(&format!("\n{}{}", app.text(key), error));
    }
    for (area, label, text) in [
        (sources, app.text("workspace.inputs"), inputs),
        (project, title.as_str(), app.text("workspace.tools").to_owned()),
        (settings, app.text("workspace.settings"), settings_text),
    ] {
        f.render_widget(
            Paragraph::new(text).wrap(Wrap { trim: false })
                .block(views::block(label.to_owned()).padding(Padding::horizontal(1))),
            area,
        );
    }
    views::status(f, footer, app, "category.back");
}

fn actions(f: &mut Frame, app: &App, title: String) {
    let [header_area, body, footer] = category_layout(f);
    header::render_header(f, header_area, app);
    let title = if app.search.is_empty() {
        title
    } else {
        format!("{title} / {}", app.search)
    };
    let items: Vec<_> = app
        .action_list()
        .into_iter()
        .map(|(_, action)| {
            let mark = if app.project.data.favorites.contains(&action.id) { " ★" } else { "" };
            let (badge, name) = if action.is_document() {
                ("PDF", action.name(app.language()))
            } else if action.is_log_capture() {
                ("LOG → PCAP", action.name(app.language()))
            } else if app.project.data.active_log().is_some() {
                action.log_query.map_or(("PCAP", action.name(app.language())), |query| ("LOG", app.text(query.label())))
            } else {
                ("", action.name(app.language()))
            };
            let source = if badge.is_empty() { String::new() } else { format!(" [{badge}]") };
            ListItem::new(format!("[{}]{source} {name}{mark}", action.id))
        })
        .collect();
    if items.is_empty() {
        let message = if app.search.is_empty() { "category.empty" } else { "search.empty" };
        f.render_widget(Paragraph::new(app.text(message)).block(views::block(title)), body);
    } else {
        views::list(f, body, title, items, app.list_index);
    }
    views::status(f, footer, app, "category.actions_hint");
}
