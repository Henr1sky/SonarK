use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap},
};

use crate::{actions::Operation, tshark::sanitize};

use super::{
    app::{App, Screen},
    header,
    input::Overlay,
    theme,
};

pub fn block(title: impl Into<Line<'static>>) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::CATEGORY_BORDER))
        .title_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))
        .title(title)
}

pub fn list(f: &mut Frame, area: Rect, title: String, items: Vec<ListItem<'_>>, selected: usize) {
    let mut state = ListState::default();
    if selected < items.len() {
        state.select(Some(selected));
    }
    let list = List::new(items)
        .block(block(title))
        .highlight_style(theme::selected_style())
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, &mut state);
}

pub fn status(f: &mut Frame, area: Rect, app: &App, hint: &'static str) {
    let text = app.status.as_ref().map_or_else(
        || app.text(hint).to_owned(),
        |(key, detail)| format!("{}{}", app.text(key), sanitize(detail)),
    );
    let color = if app.status.is_some() { theme::NOTICE } else { theme::MUTED };
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(color)).wrap(Wrap { trim: false }),
        area,
    );
}

pub fn history(f: &mut Frame, app: &App) {
    let [header_area, body, footer] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(2),
        Constraint::Length(2),
    ])
    .areas(f.area());
    header::render_header(f, header_area, app);
    let history = &app.project.data.history;
    let items = history
        .iter()
        .rev()
        .map(|entry| {
            let capture = entry.capture.file_name().unwrap_or(entry.capture.as_os_str());
            ListItem::new(format!(
                "[{}]  {}  #{}",
                entry.action,
                sanitize(&capture.to_string_lossy()),
                entry.timestamp_ms
            ))
        })
        .collect();
    if history.is_empty() {
        f.render_widget(
            Paragraph::new(app.text("history.empty")).block(block(app.text("history.title"))),
            body,
        );
    } else {
        list(f, body, app.text("history.title").into(), items, app.list_index);
    }
    status(f, footer, app, "history.hint");
}

pub fn result(f: &mut Frame, app: &App) {
    let analysis = app.analysis.as_ref().filter(|_| app.screen == Screen::Action);
    let entry = app.current_result();
    let id = analysis.map_or_else(
        || entry.map_or("", |entry| entry.action.as_str()),
        |analysis| analysis.action().id.as_str(),
    );
    let title = analysis.map_or_else(
        || app.text("history.result"),
        |analysis| {
            let action = analysis.action();
            if app.project.data.active_log().is_some() {
                if let Some(query) = action.log_query {
                    return app.text(query.label());
                }
            }
            action.name(app.language())
        },
    );
    let mut info: Vec<Line> = Vec::new();
    let has_editor = analysis.is_some_and(super::analysis::Analysis::has_editor);

    if let Some(analysis) = analysis {
        let project = &app.project.data;
        let document = analysis.action().is_document();
        let conversion = analysis.action().is_log_capture();
        let is_log = !document && !conversion && project.active_log().is_some();
        if is_log && analysis.action().log_query.is_none() {
            info.push(Line::raw(app.text("action.requires_pcap")));
        } else if let Some(path) = app.analysis_path() {
            if has_editor {
                info.push(Line::raw(app.text(action_help_key(analysis.action().id.as_str(), false))));
                info.push(Line::styled(app.text("editor.original"), Style::default().fg(theme::MUTED)));
                info.push(Line::styled(command_excerpt(&analysis.original(path), f.area().width), Style::default().fg(theme::MUTED)));
                info.push(Line::styled(app.text("editor.preview"), Style::default().fg(theme::ACCENT)));
            }
            let preview = analysis.preview(path);
            info.push(Line::raw(if has_editor { command_excerpt(&preview, f.area().width) } else { compact_command(&preview) }));
        } else {
            let missing = if document { "action.requires_pdf" }
                else if conversion { "action.requires_log" } else { "capture.missing" };
            info.push(Line::raw(app.text(missing)));
        }
        if analysis.explanation {
            let explanation = explain_action(app, analysis, is_log);
            info.push(Line::raw(explanation));
        }
        if !analysis.action().writes_files()
            && entry.is_some_and(|entry| app.analysis_path().is_none_or(|path| analysis.preview(path) != entry.command))
         {
            info.push(Line::raw(app.text("refine.previous")));
        }
    } else if let Some(entry) = entry {
        info.push(Line::raw(sanitize(&entry.command)));
    }

    let information_height = if has_editor && f.area().height >= 26 { 12 }
        else if has_editor { 9 } else if info.len() > 1 { 7 } else { 4 };
    let [header_area, details, output_area, footer] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(information_height),
        Constraint::Min(3),
        Constraint::Length(3),
    ])
    .areas(f.area());
    header::render_header(f, header_area, app);
    f.render_widget(
        Paragraph::new(info)
            .wrap(Wrap { trim: false })
            .block(block(format!(" [{id}] {title} "))),
        details,
    );

    let live = analysis.filter(|analysis| analysis.running()).map(|analysis| analysis.live_preview());
    let (text, skip, horizontal, label) = match (analysis, entry) {
        (Some(analysis), _) if analysis.running() => {
            if analysis.processing_output() {
                (app.text("run.postprocess"), 0, 0, "output.title")
            } else if let Some(text) = live.as_deref().filter(|preview| !preview.is_empty()) {
                (text, analysis.scroll, analysis.horizontal, "run.live_title")
            } else {
                (app.text("run.running"), 0, 0, "output.title")
            }
        },
        (Some(analysis), None) => (
            if analysis.diagnostics.is_empty() {
                app.text("run.ready")
            } else {
                analysis.diagnostics.as_str()
            },
            analysis.scroll,
            analysis.horizontal,
            "output.title",
        ),
        (_, Some(entry)) => (
            if entry.output.is_empty() {
                app.text("output.empty")
            } else {
                &entry.output
            },
            analysis.map_or(app.saved_scroll, |analysis| analysis.scroll),
            analysis.map_or(app.saved_horizontal, |analysis| analysis.horizontal),
            if entry.truncated { "output.truncated" } else { "output.title" },
        ),
        _ => ("", 0, 0, "output.title"),
    };
    let lines: Vec<Line> = text
        .lines()
        .skip(skip)
        .take(usize::from(output_area.height.saturating_sub(2)))
        .map(|line| Line::raw(sanitize(line).replace('\t', "    ")))
        .collect();
    f.render_widget(
        Paragraph::new(lines)
            .scroll((0, horizontal))
            .block(block(format!(" {} ", app.text(label)))),
        output_area,
    );
    let hint = if analysis.is_some_and(|analysis| analysis.running()) {
        "run.cancel_hint"
    } else if entry.is_some_and(|entry| entry.derived_capture.is_some()) {
        "log_capture.result_hint"
    } else if has_editor {
        "editor.hint"
    } else if analysis.is_some() {
        "action.hint"
    } else {
        "history.result_hint"
    };
    if let Some(error) = analysis.and_then(|analysis| analysis.pipeline_error.as_deref()) {
        f.render_widget(Paragraph::new(format!("{}{}", app.text("editor.pipeline_failed"), sanitize(error)))
            .style(Style::default().fg(theme::NOTICE)).wrap(Wrap { trim: false }), footer);
    } else {
        status(f, footer, app, hint);
    }
}

fn explain_action(app: &App, analysis: &super::analysis::Analysis, is_log: bool) -> String {
    let action = analysis.action();
    if is_log {
        return action.log_query.map_or_else(
            || app.text("action.requires_pcap").to_owned(),
            |query| format!("{}{}", app.text("log_query.explain"), app.text(query.label())),
        );
    }
    match &action.operation {
        Operation::Fields { fields, transform, .. } => {
            let selected = if analysis.has_editor() { action.editor_fields(&analysis.settings) } else { fields };
            let mut details = format!("{}{}", app.text("explain.fields"), selected.join(", "));
            if transform.is_some() {
                details.push_str(app.text("explain.aggregate"));
            }
            details
        }
        Operation::Statistics { taps } => format!("{}{}", app.text("explain.stats"), taps.join(", ")),
        Operation::CaptureInfo { .. } => app.text("explain.capture").into(),
        Operation::Objects { protocol } => format!("{}{}", app.text("explain.objects"), protocol),
        Operation::Document { .. } => app.text("explain.pdf").into(),
        Operation::Follow { .. } => app.text("explain.follow").into(),
        Operation::PayloadText { .. } => app.text("explain.payload_text").into(),
        Operation::LogCapture { .. } => app.text("explain.log_capture").into(),
        Operation::WriteCapture => app.text("explain.write_capture").into(),
        Operation::Packets { .. } => app.text("explain.packets").into(),
    }
}

fn compact_command(command: &str) -> String {
    sanitize(command).replace('\n', " ").replace('\t', "<TAB>")
}

fn command_excerpt(command: &str, width: u16) -> String {
    let command = compact_command(command);
    let width = usize::from(width.saturating_sub(5));
    if Line::raw(command.as_str()).width() <= width {
        return command;
    }
    let mut excerpt: String = command.chars().take(width.saturating_sub(1)).collect();
    while Line::raw(excerpt.as_str()).width() >= width && !excerpt.is_empty() {
        excerpt.pop();
    }
    excerpt.push('…');
    excerpt
}

fn action_help_key(id: &str, detailed: bool) -> &'static str {
    match (id, detailed) {
        ("3.7", false) => "editor.syn_ports_summary",
        ("2.2", false) => "editor.synack_summary",
        ("4.6", false) => "editor.payload_summary",
        ("3.7", true) => "editor.syn_ports_help",
        ("2.2", true) => "editor.synack_help",
        ("4.6", true) => "editor.payload_help",
        _ => "editor.generic_help",
    }
}

pub fn access_log(f: &mut Frame, app: &App) {
    let Some(log) = &app.log else { return };
    let [header_area, details, body, footer] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(5),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(f.area());
    header::render_header(f, header_area, app);
    let is_reference = app.reference_path.is_some();
    let path = app.reference_path.as_ref().or(app.project.data.access_log.as_ref())
        .map_or_else(String::new, |path| sanitize(&path.to_string_lossy()));
    let mut summary = format!("{} {}/{}", app.text("log.rows"), log.matches.len(), log.len());
    if log.truncated {
        summary.push_str(app.text("log.truncated"));
    }
    if !log.query.is_empty() {
        summary.push_str(&format!("   / {}", log.query));
    }
    let statuses = if is_reference {
        String::new()
    } else {
        log.statuses.iter().map(|(code, count)| format!("{code} × {count}"))
            .collect::<Vec<_>>().join("   ")
    };
    let details_text = vec![
        Line::styled(path, Style::default().fg(theme::ACCENT)),
        Line::raw(summary),
        Line::styled(statuses, Style::default().fg(theme::MUTED)),
    ];
    let title = if is_reference { "reference.title" } else { "log.title" };
    f.render_widget(Paragraph::new(details_text).block(block(app.text(title))), details);
    let lines: Vec<_> = log.visible(usize::from(body.height.saturating_sub(2)))
        .map(|(number, line)| {
            Line::from(vec![
                Span::styled(format!("{number:>6}  "), Style::default().fg(theme::MUTED)),
                Span::raw(line.replace('\t', "    ")),
            ])
        }).collect();
    let lines = if lines.is_empty() { vec![Line::raw(app.text("log.empty"))] } else { lines };
    f.render_widget(
        Paragraph::new(lines).scroll((0, log.horizontal)).block(block(app.text("log.entries"))),
        body,
    );
    status(f, footer, app, if is_reference { "reference.viewer_hint" } else { "log.hint" });
}

pub fn overlay(f: &mut Frame, app: &App, overlay: &Overlay) {
    let area = f.area();
    let is_list = matches!(overlay, Overlay::Files(_) | Overlay::Projects(_) | Overlay::References(_)
        | Overlay::Guide(_) | Overlay::Pending(_, _) | Overlay::Pipeline(_) | Overlay::StageBank(_, _)
        | Overlay::Command { .. } | Overlay::ActionHelp(_));
    let popup_height = if is_list { area.height.saturating_sub(6) }
        else if matches!(overlay, Overlay::Refine) { 20 }
        else if matches!(overlay, Overlay::Input(field, _) if field.edits_command()) { 16 }
        else { 11 };
    let [_, popup, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(popup_height),
        Constraint::Fill(1),
    ])
    .areas(area);
    let [_, popup, _] = Layout::horizontal([
        Constraint::Percentage(8),
        Constraint::Percentage(84),
        Constraint::Percentage(8),
    ])
    .areas(popup);
    f.render_widget(Clear, popup);
    match overlay {
        Overlay::Input(field, editor) => {
            let inner = block("").inner(popup);
            let width = inner.width.saturating_sub(2);
            let (value, cursor) = editor.visible(width);
            let error = app.status.as_ref().map_or_else(String::new, |(key, detail)| {
                format!("{}{}", app.text(key), sanitize(detail))
            });
            let mut text = vec![
                Line::styled(sanitize(&value), Style::default().bg(theme::FIELD_BACKGROUND)),
                Line::raw(""),
                Line::styled(app.text("input.hint"), Style::default().fg(theme::MUTED)),
                Line::styled(error, Style::default().fg(theme::NOTICE)),
            ];
            if matches!(field, super::input::Field::Group) {
                if let Some(analysis) = &app.analysis {
                    text.push(Line::raw(analysis.action().editor_fields(&analysis.settings).join(", ")));
                }
            }
            if field.edits_command() {
                if let (Some(analysis), Some(path)) = (&app.analysis, app.analysis_path()) {
                    if analysis.has_editor() {
                        let mut settings = analysis.settings.clone();
                        text.push(Line::styled(app.text("editor.preview"), Style::default().fg(theme::ACCENT)));
                        match field.apply_live(&mut settings, analysis.action(), &editor.text()) {
                            Ok(()) => text.push(Line::raw(compact_command(&analysis.preview_with(path, &settings)))),
                            Err(key) => text.push(Line::styled(app.text(key), Style::default().fg(theme::NOTICE))),
                        }
                        if matches!(field, super::input::Field::Stage(_)) {
                            text.push(Line::styled(app.text("pipeline.syntax"), Style::default().fg(theme::MUTED)));
                        }
                    }
                }
            }
            f.render_widget(
                Paragraph::new(text).wrap(Wrap { trim: false })
                    .block(block(if matches!(field, super::input::Field::Filter) && app.project.data.active_log().is_some() {
                        app.text("log_query.input_filter")
                    } else { app.text(field.label()) }).padding(Padding::horizontal(1))),
                popup,
            );
            if width > 0 && inner.height > 0 {
                f.set_cursor_position((inner.x.saturating_add(1 + cursor), inner.y));
            }
        }
        Overlay::Files(picker) => {
            let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(4)]).areas(popup);
            let title = format!(" {} · {} ", app.text(picker.source.title()), sanitize(&picker.directory.to_string_lossy()));
            let items = picker.entries.iter().map(|entry| {
                let name = sanitize(&entry.path.file_name().unwrap_or_default().to_string_lossy());
                if entry.directory {
                    ListItem::new(format!("{name}/")).style(Style::default().fg(theme::ACCENT))
                } else {
                    ListItem::new(name)
                }
            }).collect();
            list(f, body, title, items, picker.selected);
            status(f, footer, app, if picker.entries.is_empty() { "picker.empty" } else { "picker.files_hint" });
        }
        Overlay::Projects(picker) => {
            let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(popup);
            let items = picker.names.iter().map(|name| {
                let active = if name == &app.project.name { " ●" } else { "" };
                ListItem::new(format!("{name}{active}"))
            }).collect();
            list(f, body, app.text("picker.projects").into(), items, picker.selected);
            status(f, footer, app, "picker.projects_hint");
        }
        Overlay::Setup => {
            let Some(draft) = &app.draft else { return };
            let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(popup);
            let input = draft.input.as_ref().map_or_else(
                || app.text("setup.optional").to_owned(),
                |input| sanitize(&input.path().to_string_lossy()),
            );
            let summary = format!("{}{}\n{}{}\n{}{}",
                app.text("setup.name"), draft.name,
                app.text("setup.input"), input,
                app.text("setup.references"), draft.references.len());
            popup_text(f, body, app.text("setup.title"), summary);
            status(f, footer, app, "setup.hint");
        }
        Overlay::References(selected) => {
            let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(popup);
            let items: Vec<_> = app.project.data.references.iter()
                .map(|path| ListItem::new(sanitize(&path.to_string_lossy()))).collect();
            if items.is_empty() {
                popup_text(f, body, app.text("reference.title"), app.text("reference.empty").into());
            } else {
                list(f, body, app.text("reference.title").into(), items, *selected);
            }
            status(f, footer, app, "reference.list_hint");
        }
        Overlay::Guide(offset) => {
            let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(popup);
            let guide = app.text("tutor.guide").lines().skip(*offset).collect::<Vec<_>>().join("\n");
            popup_text(f, body, app.text("guide.title"), guide);
            f.render_widget(Paragraph::new(app.text("guide.hint")), footer);
        }
        Overlay::Pending(category, selected) => {
            let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(popup);
            let pending = crate::actions::pending(*category);
            let title = format!(" [{category}] {} ", app.text("pending.title"));
            if pending.is_empty() {
                popup_text(f, body, app.text("pending.title"), app.text("pending.empty").into());
            } else {
                let rows = pending.into_iter().map(|(id, title)| ListItem::new(format!("[{id}] {title}"))).collect();
                list(f, body, title, rows, *selected);
            }
            f.render_widget(Paragraph::new(app.text("pending.hint")), footer);
        }
        Overlay::Pipeline(selected) => {
            let Some(analysis) = &app.analysis else { return };
            let [body, command, footer] = Layout::vertical([
                Constraint::Fill(1), Constraint::Length(5), Constraint::Length(3),
            ]).areas(popup);
            let items = analysis.steps().into_iter().map(|step| {
                let marker = if step.enabled { "●" } else { "○" };
                let style = if step.enabled { Style::default() } else { Style::default().fg(theme::MUTED) };
                ListItem::new(format!("{marker}  {}   ·   {}", app.text(step.stage.key()), step.stage.editor_text())).style(style)
            }).collect();
            list(f, body, app.text("pipeline.title").into(), items, *selected);
            let preview = app.analysis_path().map_or_else(String::new, |path| compact_command(&analysis.preview(path)));
            popup_text(f, command, app.text("editor.preview"), preview);
            status(f, footer, app, "pipeline.hint");
        }
        Overlay::StageBank(_, selected) => {
            let Some(analysis) = &app.analysis else { return };
            let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(popup);
            let items = crate::tshark::pipeline::bank(analysis.action()).into_iter().map(|stage| {
                ListItem::new(format!("{}   ·   {}", app.text(stage.key()), stage.editor_text()))
            }).collect();
            list(f, body, app.text("pipeline.bank").into(), items, *selected);
            status(f, footer, app, "pipeline.bank_hint");
        }
        Overlay::ActionHelp(offset) => {
            let Some(analysis) = &app.analysis else { return };
            let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(popup);
            let action = analysis.action();
            let text = app.text(action_help_key(&action.id, true));
            let details = if analysis.has_editor() {
                app.text("editor.shared_help").to_owned()
            } else {
                let is_log = app.project.data.active_log().is_some() && !action.is_document() && !action.is_log_capture();
                explain_action(app, analysis, is_log)
            };
            let guide = format!("{text}\n\n{details}").lines().skip(*offset).collect::<Vec<_>>().join("\n");
            popup_text(f, body, app.text("editor.help_title"), guide);
            f.render_widget(Paragraph::new(app.text("editor.help_footer")), footer);
        }
        Overlay::SaveOnExit { .. } => {
            let mut help = app.text("editor.save_choices").to_owned();
            if let Some((key, detail)) = &app.status {
                help.push_str(&format!("\n{}{}", app.text(key), sanitize(detail)));
            }
            popup_text(f, popup, app.text("editor.save_question"), help);
        }
        Overlay::Command { original } => {
            let Some(analysis) = &app.analysis else { return };
            let command = app.analysis_path().map_or_else(
                || app.text("capture.missing").to_owned(),
                |path| sanitize(&if *original { analysis.original(path) } else { analysis.preview(path) }),
            );
            let kind = app.text(if *original { "editor.original" } else { "editor.preview" });
            popup_text(f, popup, app.text("editor.copy_title"),
                format!("{}\n\n{kind}\n{command}", app.text("editor.copy_hint")));
        }
        Overlay::Refine => {
            let Some(analysis) = &app.analysis else { return };
            let limit = analysis.settings.limit.map(|limit| limit.to_string())
                .unwrap_or_else(|| app.text("refine.all").into());
            let filter = if analysis.accepts_filter() {
                if app.project.data.active_log().is_some() { app.text("log_query.filter") }
                else { app.text("refine.filter") }
            } else {
                app.text("refine.no_filter")
            };
            let mode = if analysis.supports_mode() {
                format!("{}{}", app.text("refine.mode"), app.text(analysis.settings.mode.label()))
            } else {
                String::new()
            };
            let guided = analysis.settings.steps.is_none();
            let mut choices = filter.to_owned();
            if guided {
                choices.push_str(&format!("\n{}{limit}", app.text("refine.limit")));
            }
            if guided && !mode.is_empty() {
                choices.push_str(&format!("\n{mode}"));
            }
            if matches!(analysis.action().operation, Operation::Follow { .. }) {
                choices.push_str(&format!("\n{}{}", app.text("refine.stream"), analysis.settings.stream));
            }
            if app.project.data.active_log().is_none() {
                if analysis.action().accepts_tcp_options() {
                    let port = analysis.settings.tcp_port.map(|p| p.to_string()).unwrap_or_else(|| "–".into());
                    choices.push_str(&format!("\n{}{port}   {}{}", app.text("refine.port"),
                        app.text("refine.direction"), app.text(analysis.settings.direction.key())));
                    choices.push_str(&format!("\n{}{}", app.text("refine.psh"),
                        app.text(if analysis.settings.psh_only { "refine.on" } else { "refine.off" })));
                }
                if guided && !analysis.action().fields().is_empty() {
                    choices.push_str(&format!("\n{}{}", app.text("refine.group"),
                        analysis.settings.group.as_deref().unwrap_or("–")));
                }
                if guided && analysis.action().payload_field().is_some() {
                    choices.push_str(&format!("\n{}{}", app.text("refine.decode"),
                        app.text(if analysis.settings.decode_payload { "refine.on" } else { "refine.off" })));
                }
            }
            if analysis.has_editor() {
                choices.push_str(&format!("\n{}\n{}", app.text("editor.endpoint_keys"), app.text("editor.extra_keys")));
                if guided { choices.push_str(&format!("\n{}", app.text("editor.quick_limits"))); }
            }
            choices.push_str(&format!("\n{}", app.text("category.back")));
            popup_text(f, popup, app.text("refine.title"), choices);
        }
        Overlay::Export => {
            let mut choices = app.text("export.choices").to_owned();
            if let Some(format) = app.current_result().and_then(|entry| entry.native_format) {
                choices.push_str(&format!("\n{}.{}", app.text("export.native"), format.extension()));
            }
            popup_text(f, popup, app.text("export.title"), choices);
        }
    }
}

fn popup_text(f: &mut Frame, area: Rect, label: &'static str, text: String) {
    f.render_widget(
        Paragraph::new(Text::raw(text)).wrap(Wrap { trim: false })
            .block(block(label).padding(Padding::horizontal(1))),
        area,
    );
}
