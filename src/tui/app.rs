use std::{env, io, path::{Path, PathBuf}};

use ratatui::Frame;

use crate::{
    actions::{self, Action},
    config::Config,
    history::Entry,
    i18n::Language,
    log::AccessLog,
    project::{Data, ExportFormat, InputFile, Project, readable_file},
};

use super::{
    analysis::{Analysis, scroll_by},
    categories, category, help,
    input::{Editor, Field, Overlay},
    menu,
    picker::{FilePicker, ProjectPicker, Source, expand_home},
    splash, views,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Splash,
    Menu,
    Category(usize),
    Action,
    History,
    SavedResult(usize),
    Favorites,
    AccessLog,
}

#[derive(Default)]
pub struct ProjectDraft {
    pub name: String,
    pub input: Option<InputFile>,
    pub references: Vec<PathBuf>,
}

pub struct App {
    pub version: String,
    pub screen: Screen,
    pub selected: usize,
    pub list_index: usize,
    pub search: String,
    pub help_open: bool,
    pub quit: bool,
    pub project: Project,
    pub analysis: Option<Analysis>,
    pub background: Option<Analysis>,
    pub overlay: Option<Overlay>,
    pub status: Option<(&'static str, String)>,
    pub saved_scroll: usize,
    pub saved_horizontal: u16,
    pub log: Option<AccessLog>,
    pub reference_path: Option<PathBuf>,
    pub draft: Option<ProjectDraft>,
    pub log_back: Screen,
    action_back: Screen,
    config: Config,
    pub config_error: Option<(&'static str, String)>,
}

impl App {
    pub fn new(version: &str, project: String, capture: Option<&Path>) -> io::Result<Self> {
        let (config, config_error) = match Config::load() {
            Ok(config) => (config, None),
            Err(error) => (Config::default(), Some(("workspace.load_error", error.to_string()))),
        };
        let mut project = Project::open(project)?;
        if let Some(capture) = capture {
            InputFile::from_path(capture).and_then(|input| project.set_input(input)).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("{}{}: {error}", config.language.text("capture.failed"), capture.display()),
                )
            })?;
        }
        Ok(Self {
            version: version.to_owned(),
            screen: Screen::Splash,
            selected: 0,
            list_index: 0,
            search: String::new(),
            help_open: false,
            quit: false,
            project,
            analysis: None,
            background: None,
            overlay: None,
            status: None,
            saved_scroll: 0,
            saved_horizontal: 0,
            log: None,
            reference_path: None,
            draft: None,
            log_back: Screen::Menu,
            action_back: Screen::Menu,
            config,
            config_error,
        })
    }

    pub fn language(&self) -> Language {
        self.config.language
    }

    pub fn text(&self, key: &'static str) -> &'static str {
        self.language().text(key)
    }

    pub fn switch_language(&mut self) {
        self.config.language = self.language().other();
        self.config_error = self
            .config
            .save()
            .err()
            .map(|error| ("workspace.save_error", error.to_string()));
    }

    pub fn action_list(&self) -> Vec<(usize, &'static Action)> {
        let query = self.search.to_lowercase();
        let matches = |action: &Action| {
            query.is_empty()
                || action.id.contains(&query)
                || action.name(self.language()).to_lowercase().contains(&query)
                || (self.project.data.active_log().is_some() && action.log_query.is_some_and(|log_query| {
                    self.text(log_query.label()).to_lowercase().contains(&query)
                }))
        };
        match self.screen {
            Screen::Category(index) => actions::in_category(categories::number(index))
                .filter(|(_, action)| matches(action))
                .collect(),
            Screen::Favorites => actions::CATALOG
                .iter()
                .enumerate()
                .filter(|(_, action)| self.project.data.favorites.contains(&action.id))
                .filter(|(_, action)| matches(action))
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn open_action(&mut self) {
        if let Some(&(index, _)) = self.action_list().get(self.list_index) {
            self.action_back = self.screen;
            self.analysis = if self.background.as_ref().is_some_and(|job| job.index == index) {
                self.background.take()
            } else {
                Some(Analysis::new(index, self.project.data.active_log().is_some()))
            };
            self.screen = Screen::Action;
        }
    }

    pub fn request_leave(&mut self, quit: bool) {
        if self.analysis.as_ref().is_some_and(Analysis::dirty) {
            self.overlay = Some(Overlay::SaveOnExit { quit });
        } else {
            self.finish_leave(quit);
        }
    }

    pub fn finish_leave(&mut self, quit: bool) {
        if quit { self.quit = true; } else { self.leave_action(); }
    }

    pub fn apply_editor_input(&mut self, field: Field, text: String) {
        let Some(analysis) = &mut self.analysis else { return };
        let action = analysis.action();
        if let Err(key) = field.apply_live(&mut analysis.settings, action, &text) {
            self.status = Some((key, String::new()));
            self.overlay = Some(Overlay::Input(field, Editor::new(text)));
            return;
        }
        match field {
            Field::Stage(index) => {
                analysis.update_output();
                self.overlay = Some(Overlay::Pipeline(index));
            }
            Field::Limit | Field::Group => analysis.update_output(),
            _ => self.status = Some(("refine.rerun", String::new())),
        }
    }

    pub fn leave_action(&mut self) {
        self.analysis = None;
        self.status = None;
        self.screen = self.action_back;
        self.list_index = self.list_index.min(self.action_list().len().saturating_sub(1));
    }

    pub fn analysis_path(&self) -> Option<&Path> {
        if self.analysis.as_ref().is_some_and(|analysis| analysis.action().is_log_capture()) {
            self.project.data.access_log.as_deref()
        } else if self.analysis.as_ref().is_some_and(|analysis| analysis.action().is_document()) {
            self.project.data.references.iter().find(|path| {
                path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
            }).map(PathBuf::as_path)
        } else {
            self.project.data.input_path()
        }
    }

    pub fn start_analysis(&mut self) {
        if self.background.as_ref().is_some_and(Analysis::running) {
            self.status = Some(("run.background_busy", String::new()));
            return;
        }
        let is_document = self.analysis.as_ref().is_some_and(|analysis| analysis.action().is_document());
        let is_log_capture = self.analysis.as_ref().is_some_and(|analysis| analysis.action().is_log_capture());
        if !is_document && !is_log_capture && self.project.data.active_log().is_some() {
            let language = self.language();
            if let Some(analysis) = &mut self.analysis {
                self.status = if analysis.action().log_query.is_some() {
                    analysis.run_log(&mut self.project, language)
                } else {
                    Some(("action.requires_pcap", String::new()))
                };
            }
            return;
        }
        let Some(capture) = self.analysis_path().map(Path::to_path_buf) else {
            if is_document {
                self.status = Some(("action.requires_pdf", String::new()));
            } else if is_log_capture {
                self.status = Some(("action.requires_log", String::new()));
            } else {
                self.pick_file(Source::Input);
            }
            return;
        };
        self.status = self
            .analysis
            .as_mut()
            .and_then(|analysis| analysis.start(&capture, &self.project).err())
            .map(|error| {
                let key = if error.kind() == io::ErrorKind::NotFound && capture.is_file() {
                    if is_document { "run.install_pdf" }
                    else if is_log_capture { "run.install_text2pcap" }
                    else { "run.install" }
                } else {
                    "run.failed"
                };
                (key, error.to_string())
            });
    }

    pub fn background_action(&mut self) {
        self.background = self.analysis.take();
        self.leave_action();
        self.status = Some(("run.background", String::new()));
    }

    pub fn has_running_work(&self) -> bool {
        self.analysis.as_ref().is_some_and(Analysis::running)
            || self.background.as_ref().is_some_and(Analysis::running)
    }

    pub fn poll_analysis(&mut self) {
        if let Some(analysis) = &mut self.analysis {
            if let Some(status) = analysis.poll(&mut self.project) {
                self.status = Some(status);
            }
        }
        if let Some(background) = self.background.as_mut().filter(|job| job.running()) {
            let status = background.poll(&mut self.project);
            if !background.running() {
                self.status = status.or_else(|| Some(("run.background_done", background.action().id.clone())));
            }
        }
    }

    pub fn prompt(&mut self, field: Field) {
        let contents = match field {
            Field::SourcePath(source) => self.source_path(source).map(|path| path.to_string_lossy().into_owned()),
            Field::Project => self.draft.as_ref().map(|draft| draft.name.clone()),
            Field::RenameProject => Some(self.project.name.clone()),
            Field::LogSearch => self.log.as_ref().map(|log| log.query.clone()),
            Field::Search => Some(self.search.clone()),
            Field::Notes => Some(self.project.data.notes.clone()),
            Field::Filter => self.analysis.as_ref().map(|analysis| analysis.settings.filter.clone()),
            Field::BaseFilter => self.analysis.as_ref().and_then(|analysis| {
                analysis.settings.override_filter.clone().or_else(|| match &analysis.action().operation {
                    crate::actions::Operation::Fields { filter, .. } => Some(filter.clone()),
                    _ => None,
                })
            }),
            Field::OutputFields => self.analysis.as_ref().map(|analysis| {
                analysis.action().editor_fields(&analysis.settings).join(", ")
            }),
            Field::Endpoint(index) => self.analysis.as_ref().and_then(|analysis| analysis.settings.endpoints[index]).map(|ip| ip.to_string()),
            Field::Stage(index) => self.analysis.as_ref().and_then(|analysis| analysis.steps().get(index).map(|step| step.stage.editor_text())),
            Field::Stream => self.analysis.as_ref().map(|analysis| analysis.settings.stream.to_string()),
            Field::TcpPort => self.analysis.as_ref().and_then(|analysis| analysis.settings.tcp_port).map(|port| port.to_string()),
            Field::Group => self.analysis.as_ref().and_then(|analysis| analysis.settings.group.clone()),
            Field::Limit => self.analysis.as_ref().and_then(|analysis| analysis.settings.limit).map(|limit| limit.to_string()),
        };
        self.overlay = Some(Overlay::Input(field, Editor::new(contents.unwrap_or_default())));
        self.status = None;
    }

    pub fn submit(&mut self, field: Field, text: String) {
        self.status = None;
        if field.edits_command() && self.analysis.as_ref().is_some_and(Analysis::has_editor) {
            self.apply_editor_input(field, text);
            return;
        }
        match field {
            Field::BaseFilter | Field::OutputFields | Field::Endpoint(_) | Field::Stage(_) => {}
            Field::SourcePath(source) => {
                let path = expand_home(&text);
                if path.is_dir() {
                    self.show_picker(source, path);
                    if self.overlay.is_none() {
                        self.overlay = Some(Overlay::Input(field, Editor::new(text)));
                    }
                } else if !self.select_source(source, &path) {
                    self.overlay = Some(Overlay::Input(field, Editor::new(text)));
                }
            }
            Field::Project => {
                if let Some(draft) = &mut self.draft {
                    draft.name = text.trim().to_owned();
                    self.overlay = Some(Overlay::Setup);
                }
            }
            Field::RenameProject => {
                if self.has_running_work() {
                    self.status = Some(("run.background_busy", String::new()));
                    return;
                }
                if let Err(error) = self.project.rename(text.trim()) {
                    self.status = Some(("project.failed", error.to_string()));
                    self.overlay = Some(Overlay::Input(field, Editor::new(text)));
                } else {
                    self.status = Some(("project.renamed", String::new()));
                }
            }
            Field::LogSearch => {
                if let Some(log) = &mut self.log {
                    log.filter(text);
                }
            }
            Field::Search => {
                self.search = text.trim().to_owned();
                self.list_index = 0;
            }
            Field::Notes => {
                let previous = std::mem::replace(&mut self.project.data.notes, text);
                if let Err(error) = self.project.save() {
                    self.project.data.notes = previous;
                    self.status = Some(("project.failed", error.to_string()));
                }
            }
            Field::Filter => {
                if let Some(analysis) = &mut self.analysis {
                    analysis.settings.filter = text.trim().to_owned();
                    self.status = Some(("refine.rerun", String::new()));
                }
            }
            Field::TcpPort => {
                if let Some(analysis) = &mut self.analysis {
                    let input = text.trim();
                    if input.is_empty() {
                        analysis.settings.tcp_port = None;
                    } else if let Ok(port @ 1..) = input.parse::<u16>() {
                        analysis.settings.tcp_port = Some(port);
                    } else {
                        self.status = Some(("refine.invalid_port", String::new()));
                        self.overlay = Some(Overlay::Input(field, Editor::new(text)));
                        return;
                    }
                    self.status = Some(("refine.rerun", String::new()));
                }
            }
            Field::Group => {
                if let Some(analysis) = &mut self.analysis {
                    let field_name = text.trim();
                    if field_name.is_empty() {
                        analysis.settings.group = None;
                    } else if analysis.action().fields().iter().any(|field| field == field_name) {
                        analysis.settings.group = Some(field_name.to_owned());
                        analysis.settings.mode = crate::tshark::OutputMode::Count;
                        analysis.settings.decode_payload = false;
                    } else {
                        self.status = Some(("refine.invalid_group", String::new()));
                        self.overlay = Some(Overlay::Input(field, Editor::new(text)));
                        return;
                    }
                    analysis.update_output();
                }
            }
            Field::Stream => {
                if let Some(analysis) = &mut self.analysis {
                    match text.trim().parse() {
                        Ok(stream) => {
                            analysis.settings.stream = stream;
                            self.status = Some(("refine.rerun", String::new()));
                        }
                        Err(_) => {
                            self.status = Some(("refine.invalid_stream", String::new()));
                            self.overlay = Some(Overlay::Input(field, Editor::new(text)));
                        }
                    }
                }
            }
            Field::Limit => {
                if let Some(analysis) = &mut self.analysis {
                    let limit = if text.trim().is_empty() {
                        None
                    } else if let Ok(limit @ 1..) = text.trim().parse::<usize>() {
                        Some(limit)
                    } else {
                        self.status = Some(("refine.invalid_limit", String::new()));
                        self.overlay = Some(Overlay::Input(field, Editor::new(text)));
                        return;
                    };
                    analysis.settings.limit = limit;
                    analysis.update_output();
                }
            }
        }
    }

    pub fn open_project(&mut self, name: &str) -> Result<(), ()> {
        if self.has_running_work() {
            self.status = Some(("run.background_busy", String::new()));
            return Err(());
        }
        if name == self.project.name {
            self.status = None;
            return Ok(());
        }
        match Project::open(name.to_owned()) {
            Ok(project) => {
                self.activate_project(project);
                Ok(())
            }
            Err(error) => {
                self.status = Some(("project.failed", error.to_string()));
                Err(())
            }
        }
    }

    fn activate_project(&mut self, project: Project) {
        self.project = project;
        self.analysis = None;
        self.background = None;
        self.log = None;
        self.reference_path = None;
        self.saved_scroll = 0;
        self.saved_horizontal = 0;
        self.status = Some(("project.opened", self.project.name.clone()));
    }

    pub fn begin_setup(&mut self) {
        self.draft = Some(ProjectDraft::default());
        self.prompt(Field::Project);
    }

    pub fn finish_setup(&mut self) {
        if self.has_running_work() {
            self.status = Some(("run.background_busy", String::new()));
            self.overlay = Some(Overlay::Setup);
            return;
        }
        let Some(draft) = &self.draft else { return };
        let mut data = Data { references: draft.references.clone(), ..Data::default() };
        match draft.input.clone() {
            Some(InputFile::Pcap(path)) => data.capture = Some(path),
            Some(InputFile::Log(path)) => data.access_log = Some(path),
            None => {}
        }
        match Project::create(draft.name.clone(), data) {
            Ok(project) => {
                self.activate_project(project);
                self.draft = None;
            }
            Err(error) => {
                self.status = Some(("project.failed", error.to_string()));
                self.overlay = Some(Overlay::Setup);
            }
        }
    }

    pub fn pick_project(&mut self) {
        match Project::names() {
            Ok(names) => {
                let selected = names.iter().position(|name| name == &self.project.name).unwrap_or(0);
                self.overlay = Some(Overlay::Projects(ProjectPicker { names, selected }));
                self.status = None;
            }
            Err(error) => self.status = Some(("project.failed", error.to_string())),
        }
    }

    fn source_path(&self, source: Source) -> Option<&Path> {
        if let Some(draft) = &self.draft {
            return match source {
                Source::Input => draft.input.as_ref().map(InputFile::path),
                Source::Reference => draft.references.last().map(PathBuf::as_path),
            };
        }
        match source {
            Source::Input => self.project.data.input_path(),
            Source::Reference => self.project.data.references.last().map(PathBuf::as_path),
        }
    }

    pub fn pick_file(&mut self, source: Source) {
        let path = self.source_path(source).or_else(|| self.source_path(Source::Input));
        let directory = path.and_then(Path::parent)
            .filter(|p| p.is_dir()).map(Path::to_path_buf)
            .or_else(|| env::current_dir().ok()).unwrap_or_else(|| PathBuf::from("/"));
        self.show_picker(source, directory);
    }

    fn show_picker(&mut self, source: Source, directory: PathBuf) {
        let directory = directory.canonicalize();
        match directory.and_then(|directory| FilePicker::new(source, directory)) {
            Ok(picker) => {
                self.overlay = Some(Overlay::Files(picker));
                self.status = None;
            }
            Err(error) => self.status = Some(("picker.failed", error.to_string())),
        }
    }

    pub fn select_source(&mut self, source: Source, path: &Path) -> bool {
        let result = match (source, &mut self.draft) {
            (Source::Input, Some(draft)) => InputFile::from_path(path).map(|input| {
                draft.input = Some(input);
                self.overlay = Some(Overlay::Setup);
            }),
            (Source::Reference, Some(draft)) => readable_file(path).map(|path| {
                if !draft.references.contains(&path) {
                    draft.references.push(path);
                }
                self.overlay = Some(Overlay::Setup);
            }),
            (Source::Input, None) => self.attach_input(path),
            (Source::Reference, None) => self.save_reference(path),
        };
        self.status = result.err().map(|error| ("setup.file_error", error.to_string()));
        self.status.is_none()
    }

    fn attach_input(&mut self, path: &Path) -> io::Result<()> {
        let input = InputFile::from_path(path)?;
        let log = match &input {
            InputFile::Log(path) => Some(AccessLog::open(path)?),
            InputFile::Pcap(_) => None,
        };
        self.project.set_input(input)?;
        if let Some(analysis) = &mut self.analysis {
            let mut next = Analysis::new(analysis.index, log.is_some());
            if next.has_editor() && analysis.has_editor() {
                next.settings = analysis.settings.clone();
                next.baseline = analysis.baseline.clone();
            }
            *analysis = next;
        }
        if let Some(log) = log {
            self.show_text(log, None);
        } else if self.screen == Screen::AccessLog && self.reference_path.is_none() {
            self.screen = self.log_back;
            self.log = None;
        }
        Ok(())
    }

    fn save_reference(&mut self, path: &Path) -> io::Result<()> {
        let path = readable_file(path)?;
        if !self.project.data.references.contains(&path) {
            self.project.data.references.push(path);
            if let Err(error) = self.project.save() {
                self.project.data.references.pop();
                return Err(error);
            }
        }
        self.overlay = Some(Overlay::References(0));
        Ok(())
    }

    pub fn open_reference(&mut self, index: usize) -> bool {
        let Some(path) = self.project.data.references.get(index).cloned() else { return false };
        match readable_file(&path).and_then(|path| AccessLog::open(&path)) {
            Ok(log) => {
                self.show_text(log, Some(path));
                self.status = None;
                true
            }
            Err(error) => {
                self.status = Some(("reference.failed", error.to_string()));
                false
            }
        }
    }

    pub fn open_log(&mut self) {
        if let Some(path) = self.project.data.access_log.clone() {
            match self.load_log(&path) {
                Ok(()) => {
                    self.status = None;
                    return;
                }
                Err(error) => self.status = Some(("log.failed", error.to_string())),
            }
        }
        let read_error = self.status.take();
        self.pick_file(Source::Input);
        if read_error.is_some() && self.status.is_none() {
            self.status = read_error;
        }
    }

    fn load_log(&mut self, path: &Path) -> io::Result<()> {
        let path = readable_file(path)?;
        let log = AccessLog::open(&path)?;
        self.show_text(log, None);
        Ok(())
    }

    fn show_text(&mut self, log: AccessLog, reference_path: Option<PathBuf>) {
        self.log = Some(log);
        self.reference_path = reference_path;
        if self.screen != Screen::AccessLog {
            self.log_back = self.screen;
            self.screen = Screen::AccessLog;
        }
    }

    pub fn save_preset(&mut self) {
        let Some(analysis) = &mut self.analysis else { return };
        let id = analysis.action().id.clone();
        let previous = self.project.data.presets.insert(id.clone(), analysis.settings.clone());
        self.status = match self.project.save() {
            Ok(()) => {
                analysis.baseline = analysis.settings.clone();
                Some(("preset.saved", String::new()))
            },
            Err(error) => {
                match previous {
                    Some(previous) => { self.project.data.presets.insert(id, previous); }
                    None => { self.project.data.presets.remove(&id); }
                }
                Some(("project.failed", error.to_string()))
            }
        };
    }

    pub fn load_preset(&mut self) {
        let Some(analysis) = &mut self.analysis else { return };
        self.status = match self.project.data.presets.get(&analysis.action().id) {
            Some(settings) => {
                analysis.settings = settings.clone();
                analysis.baseline = settings.clone();
                analysis.update_output();
                Some(("preset.loaded", String::new()))
            }
            None => Some(("preset.missing", String::new())),
        };
    }

    pub fn toggle_favorite(&mut self) {
        let Some(analysis) = &self.analysis else { return };
        let id = &analysis.action().id;
        let previous = self.project.data.favorites.contains(id);
        self.set_favorite(id, !previous);
        self.status = match self.project.save() {
            Ok(()) => Some((if previous { "favorite.removed" } else { "favorite.added" }, String::new())),
            Err(error) => {
                self.set_favorite(id, previous);
                Some(("project.failed", error.to_string()))
            }
        };
    }

    fn set_favorite(&mut self, id: &str, favorite: bool) {
        if favorite {
            self.project.data.favorites.insert(id.to_owned());
        } else {
            self.project.data.favorites.remove(id);
        }
    }

    pub fn current_result(&self) -> Option<&Entry> {
        match self.screen {
            Screen::Action => self.analysis.as_ref()?.result.as_ref(),
            Screen::SavedResult(index) => self.project.data.history.get(index),
            _ => None,
        }
    }

    pub fn use_derived_result(&mut self) {
        if let Some((source, derived)) = self.current_result().and_then(|entry| {
            entry.derived_capture.as_ref().map(|path| (entry.capture.clone(), path.clone()))
        }) {
            self.status = Some(match self.project.activate_derived(&source, &derived) {
                Ok(()) => ("log_capture.activated", String::new()),
                Err(error) => ("project.failed", error.to_string()),
            });
        }
    }

    pub fn toggle_derived(&mut self) {
        self.status = Some(match self.project.toggle_derived() {
            Ok(true) => ("log_capture.activated", String::new()),
            Ok(false) => ("log_capture.back_to_log", String::new()),
            Err(error) => ("project.failed", error.to_string()),
        });
    }

    pub fn export(&mut self, format: ExportFormat) {
        if let Some(entry) = self.current_result() {
            self.status = Some(match self.project.export(entry, format) {
                Ok(path) => ("export.saved", path.to_string_lossy().into_owned()),
                Err(error) => ("export.failed", error.to_string()),
            });
        }
    }

    pub fn scroll_saved(&mut self, delta: isize) {
        if let Some(entry) = self.current_result() {
            self.saved_scroll = scroll_by(&entry.output, self.saved_scroll, delta);
        }
    }

    pub fn render(&self, f: &mut Frame) {
        match self.screen {
            Screen::Splash => splash::render(f, self),
            Screen::Menu => menu::render(f, self),
            Screen::Category(i) => category::render(f, self, i),
            Screen::Action | Screen::SavedResult(_) => views::result(f, self),
            Screen::History => views::history(f, self),
            Screen::Favorites => category::favorites(f, self),
            Screen::AccessLog => views::access_log(f, self),
        }
        if self.help_open {
            help::render(f, self);
        }
        if let Some(overlay) = &self.overlay {
            views::overlay(f, self, overlay);
        }
    }
}
