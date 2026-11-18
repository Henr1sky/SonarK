use std::{
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

use crate::{
    actions::{Action, CATALOG, Operation},
    history::Entry,
    i18n::Language,
    log::{AccessLog, capture as log_capture},
    project::{ArtifactDirectory, Project, readable_file, timestamp},
    tshark::{self, Invocation, Refinements, Running, pipeline},
};

pub struct Analysis {
    pub index: usize,
    pub settings: Refinements,
    pub baseline: Refinements,
    pub explanation: bool,
    pub scroll: usize,
    pub horizontal: u16,
    pub result: Option<Entry>,
    pub diagnostics: String,
    source_output: String,
    source_truncated: bool,
    display_command: Option<String>,
    pub pipeline_error: Option<String>,
    log_mode: bool,
    pending: Option<Pending>,
    post: Option<PostJob>,
}

struct Pending {
    capture: PathBuf,
    command: String,
    display_command: String,
    process: Running,
    destination: Option<ArtifactDirectory>,
}

struct PostJob {
    entry: Entry,
    worker: JoinHandle<std::io::Result<String>>,
}

impl Analysis {
    pub fn new(index: usize, log_mode: bool) -> Self {
        let action = &CATALOG[index];
        let log_mode = log_mode && !action.is_document() && !action.is_log_capture();
        let mode = if log_mode {
            action.log_query.map_or_else(Default::default, |query| query.default_mode())
        } else {
            action.default_mode()
        };
        let baseline = Refinements {
            mode,
            limit: (action.id == "3.7").then_some(10),
            ..Default::default()
        };
        Self {
            index,
            settings: baseline.clone(),
            baseline,
            explanation: false,
            scroll: 0,
            horizontal: 0,
            result: None,
            diagnostics: String::new(),
            source_output: String::new(),
            source_truncated: false,
            display_command: None,
            pipeline_error: None,
            log_mode,
            pending: None,
            post: None,
        }
    }

    pub fn action(&self) -> &'static Action {
        &CATALOG[self.index]
    }

    pub fn supports_mode(&self) -> bool {
        if self.log_mode {
            self.action().log_query.is_some()
        } else {
            matches!(self.action().operation, Operation::Fields { .. } | Operation::PayloadText { .. })
        }
    }

    pub fn accepts_filter(&self) -> bool {
        if self.log_mode {
            self.action().log_query.is_some()
        } else {
            self.action().accepts_filter()
        }
    }

    pub fn has_editor(&self) -> bool {
        !self.log_mode && self.action().has_editor()
    }

    pub fn dirty(&self) -> bool {
        self.has_editor() && self.settings != self.baseline
    }

    pub fn steps(&self) -> Vec<pipeline::Step> {
        self.settings.steps.clone().unwrap_or_else(|| pipeline::defaults(self.action(), &self.settings))
    }

    pub fn original(&self, path: &Path) -> String {
        let baseline = Self::new(self.index, self.log_mode);
        baseline.preview(path)
    }

    pub fn preview_with(&self, path: &Path, settings: &Refinements) -> String {
        let command = Invocation::new(self.action(), path, settings).preview();
        if self.has_editor() {
            let steps = settings.steps.clone().unwrap_or_else(|| pipeline::defaults(self.action(), settings));
            format!("{command}{}", pipeline::preview(&steps))
        } else {
            command
        }
    }

    pub fn preview(&self, path: &Path) -> String {
        if self.action().is_log_capture() {
            return format!("text2pcap · {}", path.display());
        }
        if self.log_mode {
            let filter = self.settings.filter.trim();
            let suffix = if filter.is_empty() { String::new() } else { format!(" / {filter}") };
            format!("HTTP access log [{}] · {}{suffix}", self.action().id, path.display())
        } else {
            self.preview_with(path, &self.settings)
        }
    }

    pub fn invocation(&self, capture: &Path) -> Invocation {
        Invocation::new(self.action(), capture, &self.settings)
    }

    pub fn running(&self) -> bool {
        self.pending.is_some() || self.post.is_some()
    }

    pub fn live_preview(&self) -> String {
        self.pending.as_ref().map_or_else(String::new, |pending| pending.process.preview())
    }

    pub fn processing_output(&self) -> bool {
        self.post.is_some()
    }

    pub fn start(&mut self, capture: &Path, project: &Project) -> std::io::Result<()> {
        if !capture.metadata()?.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Expected a regular capture file",
            ));
        }
        let destination = if self.action().writes_files() {
            Some(project.artifact_directory(&self.action().id)?)
        } else {
            None
        };
        let invocation = match (&self.action().operation, &destination) {
            (Operation::LogCapture { raw }, Some(folder)) => {
                let format = log_capture::prepare(capture, &folder.path, *raw)?;
                Invocation::import_log(format, &folder.path)
            }
            (_, Some(folder)) => Invocation::with_destination(self.action(), capture, &self.settings, &folder.path),
            (_, None) => self.invocation(capture),
        };
        self.pending = Some(Pending {
            capture: capture.to_owned(),
            command: if self.has_editor() { self.preview(capture) } else { invocation.preview() },
            display_command: invocation.preview(),
            process: invocation.start()?,
            destination,
        });
        self.result = None;
        self.pipeline_error = None;
        self.diagnostics.clear();
        self.source_output.clear();
        self.scroll = 0;
        self.horizontal = 0;
        self.explanation = false;
        Ok(())
    }

    pub fn run_log(&mut self, project: &mut Project, language: Language) -> Option<(&'static str, String)> {
        let query = self.action().log_query?;
        let path = project.data.access_log.clone()?;
        let (text, truncated) = match readable_file(&path).and_then(|path| {
            let log = AccessLog::open(&path)?;
            Ok((query.run(&log, &self.settings.filter, language)?, log.truncated))
        }) {
            Ok(result) => result,
            Err(error) => return Some(("run.failed", error.to_string())),
        };
        self.diagnostics.clear();
        self.scroll = 0;
        self.horizontal = 0;
        self.save_result(project, path.clone(), self.preview(&path), text, truncated, None)
    }

    pub fn cancel(&mut self) {
        self.pending = None;
        self.post = None;
    }

    pub fn poll(&mut self, project: &mut Project) -> Option<(&'static str, String)> {
        if let Some(post) = &self.post {
            if !post.worker.is_finished() {
                return None;
            }
            let PostJob { mut entry, worker } = self.post.take().expect("active post-processing");
            let output = worker.join().unwrap_or_else(|_| Err(std::io::Error::other("Post-processing stopped unexpectedly")));
            return match output {
                Ok(output) => {
                    let (output, cut) = tshark::limit_text(tshark::sanitize(&output));
                    entry.output = output;
                    entry.truncated |= cut;
                    let saved = project.record(entry.clone());
                    self.result = Some(entry);
                    saved.err().map(|error| ("project.failed", error.to_string()))
                }
                Err(error) => {
                    self.pipeline_error = Some(error.to_string());
                    self.diagnostics = error.to_string();
                    Some(("editor.pipeline_failed", error.to_string()))
                }
            };
        }
        let pending = self.pending.as_mut()?;
        let completed = match pending.process.poll() {
            Ok(Some(completed)) => completed,
            Ok(None) => return None,
            Err(error) => {
                self.cancel();
                return Some(("run.failed", error.to_string()));
            }
        };
        let pending = self.pending.take().expect("active invocation");
        if !completed.status.success() {
            self.diagnostics = completed.diagnostics;
            return Some(("run.failed", completed.status.to_string()));
        }
        let mut derived_capture = None;
        let text = match pending.destination {
            Some(directory) => {
                if self.action().is_log_capture() {
                    let derived = directory.path.join(log_capture::OUTPUT);
                    if !derived.is_file() {
                        return Some(("run.failed", completed.diagnostics));
                    }
                    if let Err(error) = std::fs::remove_file(directory.path.join(log_capture::HEX_FILE)) {
                        return Some(("run.failed", error.to_string()));
                    }
                    derived_capture = Some(derived);
                }
                match directory.finish() {
                    Ok(text) => text,
                    Err(error) => return Some(("run.failed", error.to_string())),
                }
            },
            None => match self.action().operation {
                Operation::PayloadText { format } => tshark::payload_text(&completed.text, format),
                Operation::Fields { transform: Some(transform), .. } => tshark::transform(&completed.text, transform),
                _ => completed.text,
            },
        };
        self.display_command = Some(pending.display_command);
        self.save_result(project, pending.capture, pending.command, text, completed.truncated, derived_capture)
    }

    fn save_result(
        &mut self,
        project: &mut Project,
        path: PathBuf,
        command: String,
        source: String,
        source_truncated: bool,
        derived_capture: Option<PathBuf>,
    ) -> Option<(&'static str, String)> {
        self.source_output = source;
        self.source_truncated = source_truncated;
        let native_format = self.native_format(source_truncated);
        if self.has_editor() {
            let steps = self.steps();
            let output = self.source_output.clone();
            let entry = Entry {
                action: self.action().id.clone(),
                capture: path,
                timestamp_ms: timestamp(),
                command,
                output: String::new(),
                truncated: source_truncated,
                derived_capture,
                native_format: None,
            };
            self.post = Some(PostJob { entry, worker: thread::spawn(move || pipeline::run(&output, &steps)) });
            return None;
        }
        let (output, truncated) = match self.format_output(native_format.is_some()) {
            Ok(result) => result,
            Err(error) => {
                self.pipeline_error = Some(error.to_string());
                self.diagnostics = error.to_string();
                return Some(("editor.pipeline_failed", error.to_string()));
            }
        };
        let entry = Entry {
            action: self.action().id.clone(),
            capture: path,
            timestamp_ms: timestamp(),
            command,
            output,
            truncated,
            derived_capture,
            native_format,
        };
        let saved = project.record(entry.clone());
        self.result = Some(entry);
        saved.err().map(|error| ("project.failed", error.to_string()))
    }

    fn native_format(&self, truncated: bool) -> Option<crate::actions::PacketFormat> {
        if self.log_mode || truncated || self.settings.limit.is_some() {
            return None;
        }
        match self.action().operation {
            Operation::Packets { format } => Some(format),
            _ => None,
        }
    }

    fn format_output(&self, native: bool) -> std::io::Result<(String, bool)> {
        if self.has_editor() {
            let text = pipeline::run(&self.source_output, &self.steps())?;
            let (formatted, cut) = tshark::limit_text(tshark::sanitize(&text));
            return Ok((formatted, self.source_truncated || cut));
        }
        if native {
            Ok((self.source_output.clone(), self.source_truncated))
        } else {
            let selection = if self.log_mode {
                self.source_output.clone()
            } else {
                tshark::format_fields(&self.source_output, self.action(), &self.settings)
            };
            let formatted = tshark::refine(&tshark::sanitize(&selection), &self.settings);
            let (formatted, cut) = tshark::limit_text(formatted);
            Ok((formatted, self.source_truncated || cut))
        }
    }

    pub fn update_output(&mut self) {
        let format = self.native_format(self.source_truncated);
        if self.result.is_some() {
            let (output, truncated) = match self.format_output(format.is_some()) {
                Ok(result) => result,
                Err(error) => {
                    self.pipeline_error = Some(error.to_string());
                    return;
                }
            };
            self.pipeline_error = None;
            let command = self.result.as_ref().and_then(|entry| {
                (self.has_editor() && self.matches_source(&entry.capture))
                    .then(|| self.preview(&entry.capture))
            });
            let result = self.result.as_mut().expect("a result is available");
            result.output = output;
            if let Some(command) = command {
                result.command = command;
            }
            result.truncated = truncated;
            result.native_format = format;
            self.scroll = 0;
        }
    }

    pub fn matches_source(&self, path: &Path) -> bool {
        self.display_command.as_ref().is_some_and(|command| *command == self.invocation(path).preview())
    }

    pub fn move_output(&mut self, delta: isize) {
        if self.running() {
            self.scroll = scroll_by(&self.live_preview(), self.scroll, delta);
            return;
        }
        let output = self
            .result
            .as_ref()
            .map_or(self.diagnostics.as_str(), |entry| entry.output.as_str());
        self.scroll = scroll_by(output, self.scroll, delta);
    }
}

pub fn scroll_by(output: &str, offset: usize, delta: isize) -> usize {
    offset
        .saturating_add_signed(delta)
        .min(output.lines().count().saturating_sub(1))
}
