mod analysis;
mod app;
mod categories;
mod clipboard;
mod category;
mod header;
mod help;
mod input;
mod menu;
mod panic;
mod picker;
mod splash;
mod theme;
mod views;

use std::{
    io::{self, IsTerminal},
    path::Path,
    time::Duration,
};

use crossterm::{
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::{App, Screen};
use input::{Editor, Field, Overlay, is_backspace};
use picker::{FilePicker, ProjectPicker, Source};
use crate::tshark::pipeline;

pub fn run(version: &str, project: String, capture: Option<&Path>) -> io::Result<()> {
    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        let language = crate::config::Config::load().unwrap_or_default().language;
        return Err(io::Error::other(language.text("terminal.required")));
    }
    let mut app = App::new(version, project, capture)?;
    panic::install_hook();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let result = (|| {
        execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(&mut stdout))?;
        terminal.clear()?;
        main_loop(&mut terminal, &mut app)
    })();
    let raw_mode = disable_raw_mode();
    let screen = execute!(stdout, DisableBracketedPaste, LeaveAlternateScreen, crossterm::cursor::Show);
    result.and(raw_mode).and(screen)
}

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<&mut io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while !app.quit {
        app.poll_analysis();
        terminal.draw(|frame| app.render(frame))?;
        if app.has_running_work() && !event::poll(Duration::from_millis(125))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) => handle_key(app, key),
            Event::Paste(text) => {
                if let Some(Overlay::Input(_, buffer)) = &mut app.overlay {
                    buffer.insert(&text);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn is_running(app: &App) -> bool {
    app.analysis.as_ref().is_some_and(analysis::Analysis::running)
}

fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind == KeyEventKind::Release {
        return;
    }
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    if control && key.code == KeyCode::Char('c') {
        if let Some(analysis) = app.analysis.as_mut().filter(|analysis| analysis.running()) {
            analysis.cancel();
            app.status = Some(("run.cancelled", String::new()));
        } else if let Some(background) = app.background.as_mut().filter(|job| job.running()) {
            background.cancel();
            app.status = Some(("run.cancelled", String::new()));
        } else if app.screen == Screen::Action {
            app.request_leave(true);
        } else {
            app.quit = true;
        }
        return;
    }
    if let Some(overlay) = app.overlay.take() {
        handle_overlay(app, key, overlay);
        return;
    }
    if key.code == KeyCode::F(1) {
        app.help_open = false;
        app.overlay = Some(Overlay::Guide(0));
        return;
    }
    if key.code == KeyCode::Char('?') {
        app.help_open = !app.help_open;
        return;
    }
    if app.help_open {
        app.help_open = false;
        return;
    }
    if app.screen == Screen::Splash {
        app.screen = Screen::Menu;
        return;
    }
    if key.code == KeyCode::Char('q') {
        if app.screen == Screen::Action { app.request_leave(true); }
        else { app.quit = true; }
        return;
    }
    if key.code == KeyCode::Char('p') && !is_running(app) {
        app.pick_file(Source::Input);
        return;
    }
    if key.code == KeyCode::Char('g') && !is_running(app) && app.screen != Screen::AccessLog {
        app.open_log();
        return;
    }
    match app.screen {
        Screen::Menu => match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected = (app.selected + categories::COUNT - 1) % categories::COUNT;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.selected = (app.selected + 1) % categories::COUNT;
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                if app.selected == 21 {
                    app.overlay = Some(Overlay::Guide(0));
                } else {
                    open_list(app, Screen::Category(app.selected));
                }
            }
            KeyCode::Char('0') => app.quit = true,
            _ => {}
        },
        Screen::Category(index) if index == categories::WORKSPACE_INDEX => {
            match key.code {
                KeyCode::Enter if key.kind == KeyEventKind::Press => app.switch_language(),
                KeyCode::Char('n') => app.pick_project(),
                KeyCode::Char('u') => app.overlay = Some(Overlay::References(0)),
                KeyCode::Char('v') => app.toggle_derived(),
                KeyCode::Char('r') => app.prompt(Field::RenameProject),
                KeyCode::Char('m') => app.prompt(Field::Notes),
                KeyCode::Char('y') => open_list(app, Screen::History),
                KeyCode::Char('f') => open_list(app, Screen::Favorites),
                code if back(code) => open_list(app, Screen::Menu),
                _ => {}
            }
        }
        Screen::Category(_) | Screen::Favorites => match key.code {
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.open_action(),
            KeyCode::Char('i') => {
                if let Screen::Category(index) = app.screen {
                    app.overlay = Some(Overlay::Pending(categories::number(index), 0));
                }
            }
            KeyCode::Char('/') => app.prompt(Field::Search),
            code if back(code) => {
                let parent = if app.screen == Screen::Favorites {
                    Screen::Category(categories::WORKSPACE_INDEX)
                } else {
                    Screen::Menu
                };
                open_list(app, parent);
            }
            code => {
                let len = app.action_list().len();
                move_list(&mut app.list_index, len, code);
            }
        },
        Screen::Action => handle_action(app, key),
        Screen::History => match key.code {
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                if let Some(index) = app.project.data.history.len().checked_sub(app.list_index + 1) {
                    app.saved_scroll = 0;
                    app.saved_horizontal = 0;
                    app.status = None;
                    app.screen = Screen::SavedResult(index);
                }
            }
            code if back(code) => open_list(app, Screen::Category(categories::WORKSPACE_INDEX)),
            code => move_list(&mut app.list_index, app.project.data.history.len(), code),
        },
        Screen::SavedResult(_) => match key.code {
            KeyCode::Esc | KeyCode::Char('h' | '0') => {
                app.screen = Screen::History;
                app.status = None;
            }
            KeyCode::Char('x') => app.overlay = Some(Overlay::Export),
            KeyCode::Char('o') => app.use_derived_result(),
            KeyCode::Left => app.saved_horizontal = app.saved_horizontal.saturating_sub(8),
            KeyCode::Right => app.saved_horizontal = app.saved_horizontal.saturating_add(8),
            code => app.scroll_saved(scroll_delta(code)),
        },
        Screen::AccessLog => match key.code {
            KeyCode::Char('o') => {
                if app.reference_path.is_some() {
                    app.overlay = Some(Overlay::References(0));
                } else {
                    app.pick_file(Source::Input);
                }
            }
            KeyCode::Char('/') => app.prompt(Field::LogSearch),
            KeyCode::Esc | KeyCode::Char('h' | '0') => {
                app.screen = app.log_back;
                app.status = None;
            }
            code => {
                if let Some(log) = &mut app.log {
                    match code {
                        KeyCode::Left => log.horizontal = log.horizontal.saturating_sub(8),
                        KeyCode::Right => log.horizontal = log.horizontal.saturating_add(8),
                        _ => move_list(&mut log.scroll, log.matches.len(), code),
                    }
                }
            }
        },
        Screen::Splash => {}
    }
}

fn handle_action(app: &mut App, key: KeyEvent) {
    let editor = app.analysis.as_ref().is_some_and(analysis::Analysis::has_editor);
    if key.code == KeyCode::Char('h') {
        app.overlay = Some(Overlay::ActionHelp(0));
        return;
    }
    if is_running(app) {
        match key.code {
            KeyCode::Char('B') => app.background_action(),
            KeyCode::Esc => {
                if let Some(analysis) = &mut app.analysis {
                    analysis.cancel();
                    app.status = Some(("run.cancelled", String::new()));
                }
            }
            code => scroll_analysis(app, code),
        }
        return;
    }
    match key.code {
        KeyCode::Enter if key.kind == KeyEventKind::Press => app.start_analysis(),
        KeyCode::Char('a') => app.overlay = Some(Overlay::Refine),
        KeyCode::Char('t') if editor => app.overlay = Some(Overlay::Pipeline(0)),
        KeyCode::Char('c') if editor => app.overlay = Some(Overlay::Command { original: false }),
        KeyCode::Char('x') if app.current_result().is_some() => app.overlay = Some(Overlay::Export),
        KeyCode::Char('o') => app.use_derived_result(),
        KeyCode::Char('s') if key.kind == KeyEventKind::Press => app.save_preset(),
        KeyCode::Char('b') => app.load_preset(),
        KeyCode::Char('f') if key.kind == KeyEventKind::Press => app.toggle_favorite(),
        KeyCode::Char('R') if editor => {
            if let Some(analysis) = &mut app.analysis {
                let original = analysis::Analysis::new(analysis.index, false);
                analysis.settings = original.settings;
                analysis.update_output();
                app.status = Some(("editor.restored", String::new()));
            }
        }
        KeyCode::Esc | KeyCode::Char('0') => app.request_leave(false),
        KeyCode::Char('e') if key.kind == KeyEventKind::Press => {
            if let Some(analysis) = &mut app.analysis {
                analysis.explanation = !analysis.explanation;
            }
        }
        code => scroll_analysis(app, code),
    }
}

fn scroll_analysis(app: &mut App, code: KeyCode) {
    let Some(analysis) = &mut app.analysis else { return };
    match code {
        KeyCode::Left => analysis.horizontal = analysis.horizontal.saturating_sub(8),
        KeyCode::Right => analysis.horizontal = analysis.horizontal.saturating_add(8),
        code => analysis.move_output(scroll_delta(code)),
    }
}

fn handle_overlay(app: &mut App, key: KeyEvent, overlay: Overlay) {
    if key.code == KeyCode::Esc {
        match overlay {
            Overlay::Input(Field::Stage(index), _) | Overlay::StageBank(index, _) => {
                app.overlay = Some(Overlay::Pipeline(index));
            }
            Overlay::Setup => {
                app.draft = None;
                app.pick_project();
            }
            Overlay::Input(Field::Project | Field::SourcePath(_), _) | Overlay::Files(_)
                if app.draft.is_some() =>
            {
                app.status = None;
                app.overlay = Some(Overlay::Setup);
            }
            _ => {}
        }
        return;
    }
    match overlay {
        Overlay::Input(field, mut editor) => {
            if key.code == KeyCode::Enter && key.kind == KeyEventKind::Press {
                app.submit(field, editor.into_text());
            } else {
                editor.edit(key);
                app.overlay = Some(Overlay::Input(field, editor));
            }
        }
        Overlay::Files(picker) => handle_file_picker(app, key, picker),
        Overlay::Projects(picker) => handle_project_picker(app, key, picker),
        Overlay::Setup => match key.code {
            KeyCode::Enter if key.kind == KeyEventKind::Press => app.finish_setup(),
            KeyCode::Char('n') => app.prompt(Field::Project),
            KeyCode::Char('p') => setup_file_picker(app, Source::Input),
            KeyCode::Char('a') => setup_file_picker(app, Source::Reference),
            KeyCode::Char('x') => {
                if let Some(draft) = &mut app.draft {
                    draft.input = None;
                    draft.references.clear();
                }
                app.status = None;
                app.overlay = Some(Overlay::Setup);
            }
            _ => app.overlay = Some(Overlay::Setup),
        },
        Overlay::References(mut selected) => {
            match key.code {
                KeyCode::Char('a') => {
                    app.pick_file(Source::Reference);
                    return;
                }
                KeyCode::Enter if key.kind == KeyEventKind::Press => {
                    if app.open_reference(selected) {
                        return;
                    }
                }
                code => move_list(&mut selected, app.project.data.references.len(), code),
            }
            app.overlay = Some(Overlay::References(selected));
        }
        Overlay::Guide(mut offset) => {
            if key.code != KeyCode::F(1) {
                move_list(&mut offset, app.text("tutor.guide").lines().count(), key.code);
                app.overlay = Some(Overlay::Guide(offset));
            }
        }
        Overlay::Pending(category, mut selected) => {
            if key.code != KeyCode::Left {
                move_list(&mut selected, crate::actions::pending(category).len(), key.code);
                app.overlay = Some(Overlay::Pending(category, selected));
            }
        }
        Overlay::Pipeline(selected) => handle_pipeline(app, key, selected),
        Overlay::StageBank(index, mut selected) => {
            let Some(analysis) = &mut app.analysis else { return };
            let bank = pipeline::bank(analysis.action());
            if key.code == KeyCode::Enter && key.kind == KeyEventKind::Press {
                let mut steps = analysis.steps();
                if steps.len() >= 16 {
                    app.status = Some(("pipeline.too_many", String::new()));
                    app.overlay = Some(Overlay::Pipeline(index));
                    return;
                }
                if let Some(stage) = bank.get(selected) {
                    let insertion = (index + 1).min(steps.len());
                    steps.insert(insertion, pipeline::enabled(stage.clone()));
                    analysis.settings.steps = Some(steps);
                    analysis.update_output();
                    app.overlay = Some(Overlay::Pipeline(insertion));
                }
            } else {
                move_list(&mut selected, bank.len(), key.code);
                app.overlay = Some(Overlay::StageBank(index, selected));
            }
        }
        Overlay::ActionHelp(mut offset) => {
            if key.code != KeyCode::Char('h') {
                move_list(&mut offset, 30, key.code);
                app.overlay = Some(Overlay::ActionHelp(offset));
            }
        }
        Overlay::SaveOnExit { quit } => match key.code {
            KeyCode::Char('s') => {
                app.save_preset();
                if app.status.as_ref().is_some_and(|(key, _)| *key == "preset.saved") {
                    app.finish_leave(quit);
                } else {
                    app.overlay = Some(Overlay::SaveOnExit { quit });
                }
            }
            KeyCode::Char('d') => app.finish_leave(quit),
            _ => app.overlay = Some(Overlay::SaveOnExit { quit }),
        },
        Overlay::Command { original } => {
            if key.code == KeyCode::Char('r') {
                app.overlay = Some(Overlay::Command { original: !original });
            } else if key.code == KeyCode::Char('c') {
                if let (Some(path), Some(analysis)) = (app.analysis_path(), &app.analysis) {
                    let preview = if original { analysis.original(path) } else { analysis.preview(path) };
                    app.status = Some(match clipboard::copy(&preview) {
                        Ok(()) => ("editor.copied", String::new()),
                        Err(error) => ("editor.copy_failed", error.to_string()),
                    });
                }
            } else {
                app.overlay = Some(Overlay::Command { original });
            }
        }
        Overlay::Refine => {
            let log_mode = app.project.data.active_log().is_some();
            let Some(analysis) = &mut app.analysis else { return };
            let guided = analysis.settings.steps.is_none();
            match key.code {
                KeyCode::Char('t') if analysis.has_editor() => app.overlay = Some(Overlay::Pipeline(0)),
                KeyCode::Char('e') if analysis.has_editor() => app.prompt(Field::BaseFilter),
                KeyCode::Char('E') if analysis.has_editor() => app.prompt(Field::OutputFields),
                KeyCode::Char('1') if analysis.has_editor() => app.prompt(Field::Endpoint(0)),
                KeyCode::Char('2') if analysis.has_editor() => app.prompt(Field::Endpoint(1)),
                KeyCode::Char('5' | '6' | '7') if analysis.has_editor() && guided => {
                    analysis.settings.limit = Some(match key.code {
                        KeyCode::Char('5') => 10,
                        KeyCode::Char('6') => 20,
                        _ => 50,
                    });
                    analysis.update_output();
                    app.overlay = Some(Overlay::Refine);
                }
                KeyCode::Char('f') if analysis.accepts_filter() => app.prompt(Field::Filter),
                KeyCode::Char('n') if guided => app.prompt(Field::Limit),
                KeyCode::Char('p') if !log_mode && analysis.action().accepts_tcp_options() => app.prompt(Field::TcpPort),
                KeyCode::Char('g') if guided && !log_mode && !analysis.action().fields().is_empty() => app.prompt(Field::Group),
                KeyCode::Char('d') if guided && !log_mode && analysis.action().payload_field().is_some() => {
                    analysis.settings.decode_payload = !analysis.settings.decode_payload;
                    if analysis.settings.decode_payload {
                        analysis.settings.group = None;
                    }
                    analysis.update_output();
                    app.overlay = Some(Overlay::Refine);
                }
                KeyCode::Char('r') if !log_mode && analysis.action().accepts_tcp_options() => {
                    analysis.settings.direction = analysis.settings.direction.next();
                    app.status = Some(("refine.rerun", String::new()));
                    app.overlay = Some(Overlay::Refine);
                }
                KeyCode::Char('u') if !log_mode && analysis.action().accepts_tcp_options() => {
                    analysis.settings.psh_only = !analysis.settings.psh_only;
                    app.status = Some(("refine.rerun", String::new()));
                    app.overlay = Some(Overlay::Refine);
                }
                KeyCode::Char('s') if matches!(analysis.action().operation, crate::actions::Operation::Follow { .. }) => app.prompt(Field::Stream),
                KeyCode::Char('m') if guided && analysis.supports_mode() => {
                    analysis.settings.mode = analysis.settings.mode.next();
                    analysis.update_output();
                    app.overlay = Some(Overlay::Refine);
                }
                code if back(code) => {}
                _ => app.overlay = Some(Overlay::Refine),
            }
        }
        Overlay::Export => {
            if key.code == KeyCode::Char('5')
                && app.current_result().is_some_and(|entry| entry.native_format.is_some())
            {
                app.export(crate::project::ExportFormat::Native);
                return;
            }
            if let KeyCode::Char(key) = key.code {
                if let Some(format) = input::export_key(key) {
                    app.export(format);
                    return;
                }
            }
            app.overlay = Some(Overlay::Export);
        }
    }
}

fn handle_pipeline(app: &mut App, key: KeyEvent, mut selected: usize) {
    let Some(analysis) = &mut app.analysis else { return };
    let mut steps = analysis.steps();
    let changed = match key.code {
        KeyCode::Char('a') => {
            app.overlay = Some(Overlay::StageBank(selected, 0));
            return;
        }
        KeyCode::Char('e') | KeyCode::Enter if selected < steps.len() => {
            app.prompt(Field::Stage(selected));
            return;
        }
        KeyCode::Char('r') => {
            analysis.settings.steps = None;
            analysis.update_output();
            app.overlay = Some(Overlay::Pipeline(0));
            return;
        }
        KeyCode::Char(' ') if selected < steps.len() => {
            steps[selected].enabled = !steps[selected].enabled;
            true
        }
        KeyCode::Char('d') if selected < steps.len() => {
            steps.remove(selected);
            selected = selected.min(steps.len().saturating_sub(1));
            true
        }
        KeyCode::Char('<') if selected > 0 && selected < steps.len() => {
            steps.swap(selected, selected - 1);
            selected -= 1;
            true
        }
        KeyCode::Char('>') if selected + 1 < steps.len() => {
            steps.swap(selected, selected + 1);
            selected += 1;
            true
        }
        code => {
            move_list(&mut selected, steps.len(), code);
            false
        }
    };
    if changed {
        analysis.settings.steps = Some(steps);
        analysis.update_output();
    }
    app.overlay = Some(Overlay::Pipeline(selected));
}

fn setup_file_picker(app: &mut App, source: Source) {
    app.pick_file(source);
    if app.overlay.is_none() {
        app.overlay = Some(Overlay::Setup);
    }
}

fn handle_file_picker(app: &mut App, key: KeyEvent, mut picker: FilePicker) {
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    let result = match key.code {
        KeyCode::Char('t') => {
            let mut path = picker.directory.to_string_lossy().into_owned();
            if !path.ends_with('/') {
                path.push('/');
            }
            app.status = None;
            app.overlay = Some(Overlay::Input(Field::SourcePath(picker.source), Editor::new(path)));
            return;
        }
        KeyCode::Char('~') => match std::env::var_os("HOME") {
            Some(home) => picker.open(home.into()),
            None => Ok(()),
        },
        KeyCode::Char('d') => match std::env::var_os("HOME") {
            Some(home) => picker.open(std::path::PathBuf::from(home).join("Downloads")),
            None => Ok(()),
        },
        KeyCode::Char('w') => std::env::current_dir().and_then(|path| picker.open(path)),
        KeyCode::Char('a') => picker.toggle_all(),
        KeyCode::Left => picker.parent(),
        code if is_backspace(code, control) => picker.parent(),
        KeyCode::Enter | KeyCode::Right if key.kind == KeyEventKind::Press => {
            match picker.entries.get(picker.selected) {
                Some(entry) if entry.directory => picker.open(entry.path.clone()),
                Some(entry) => {
                    if app.select_source(picker.source, &entry.path) {
                        return;
                    }
                    app.overlay = Some(Overlay::Files(picker));
                    return;
                }
                None => Ok(()),
            }
        }
        code => {
            move_list(&mut picker.selected, picker.entries.len(), code);
            Ok(())
        }
    };
    app.status = result.err().map(|error| ("picker.failed", error.to_string()));
    app.overlay = Some(Overlay::Files(picker));
}

fn handle_project_picker(app: &mut App, key: KeyEvent, mut picker: ProjectPicker) {
    match key.code {
        KeyCode::Char('c') => {
            app.begin_setup();
            return;
        }
        KeyCode::Char('r') => {
            app.prompt(Field::RenameProject);
            return;
        }
        KeyCode::Left => return,
        code if is_backspace(code, key.modifiers.contains(KeyModifiers::CONTROL)) => return,
        KeyCode::Enter if key.kind == KeyEventKind::Press => {
            if let Some(name) = picker.names.get(picker.selected) {
                if app.open_project(name).is_ok() {
                    return;
                }
            }
        }
        code => {
            move_list(&mut picker.selected, picker.names.len(), code);
            app.status = None;
        }
    }
    app.overlay = Some(Overlay::Projects(picker));
}

fn open_list(app: &mut App, screen: Screen) {
    app.screen = screen;
    app.list_index = 0;
    app.search.clear();
    app.status = None;
}

fn back(code: KeyCode) -> bool {
    matches!(code, KeyCode::Esc | KeyCode::Left | KeyCode::Char('h' | '0'))
}

fn move_list(index: &mut usize, len: usize, code: KeyCode) {
    *index = index.saturating_add_signed(scroll_delta(code)).min(len.saturating_sub(1));
}

fn scroll_delta(code: KeyCode) -> isize {
    match code {
        KeyCode::Down | KeyCode::Char('j') => 1,
        KeyCode::Up | KeyCode::Char('k') => -1,
        KeyCode::PageDown => 10,
        KeyCode::PageUp => -10,
        _ => 0,
    }
}
