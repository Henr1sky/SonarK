use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::text::Line;

use crate::{actions::Action, project::ExportFormat, tshark::{Refinements, pipeline::Stage}};

use super::picker::{FilePicker, ProjectPicker, Source};

#[derive(Clone, Copy)]
pub enum Field {
    SourcePath(Source),
    Project,
    RenameProject,
    Search,
    LogSearch,
    Filter,
    BaseFilter,
    OutputFields,
    Endpoint(usize),
    Stage(usize),
    Limit,
    Stream,
    TcpPort,
    Group,
    Notes,
}

impl Field {
    pub fn label(self) -> &'static str {
        match self {
            Self::SourcePath(Source::Input) => "input.source",
            Self::SourcePath(Source::Reference) => "input.reference",
            Self::Project => "input.project",
            Self::RenameProject => "input.rename",
            Self::Search => "input.search",
            Self::LogSearch => "input.log_search",
            Self::Filter => "input.filter",
            Self::BaseFilter => "editor.base_filter",
            Self::OutputFields => "editor.output_fields",
            Self::Endpoint(0) => "editor.source_ip",
            Self::Endpoint(_) => "editor.destination_ip",
            Self::Stage(_) => "pipeline.edit_title",
            Self::Limit => "input.limit",
            Self::Stream => "input.stream",
            Self::TcpPort => "input.tcp_port",
            Self::Group => "input.group",
            Self::Notes => "input.notes",
        }
    }
}

pub enum Overlay {
    Input(Field, Editor),
    Files(FilePicker),
    Projects(ProjectPicker),
    Setup,
    References(usize),
    Guide(usize),
    Pending(u16, usize),
    Refine,
    Pipeline(usize),
    StageBank(usize, usize),
    ActionHelp(usize),
    SaveOnExit { quit: bool },
    Command { original: bool },
    Export,
}

pub struct Editor {
    text: Vec<char>,
    cursor: usize,
}

impl Editor {
    pub fn new(text: String) -> Self {
        let text: Vec<char> = text.chars().collect();
        let cursor = text.len();
        Self { text, cursor }
    }

    pub fn text(&self) -> String { self.text.iter().collect() }

    pub fn into_text(self) -> String {
        self.text.into_iter().collect()
    }

    pub fn insert(&mut self, text: &str) {
        let start = self.cursor;
        self.text.splice(start..start, text.chars().map(|ch| {
            if ch.is_control() { ' ' } else { ch }
        }));
        self.cursor += text.chars().count();
    }

    pub fn edit(&mut self, key: KeyEvent) {
        let control = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.text.len()),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.text.len(),
            KeyCode::Char('a') if control => self.cursor = 0,
            KeyCode::Char('e') if control => self.cursor = self.text.len(),
            KeyCode::Char('u') if control => {
                self.text.clear();
                self.cursor = 0;
            }
            code if is_backspace(code, control) => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.text.remove(self.cursor);
                }
            }
            KeyCode::Delete if self.cursor < self.text.len() => {
                self.text.remove(self.cursor);
            }
            KeyCode::Char(ch)
                if !ch.is_control()
                    && !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.text.insert(self.cursor, ch);
                self.cursor += 1;
            }
            _ => {}
        }
    }

    pub fn visible(&self, width: u16) -> (String, u16) {
        let width = usize::from(width).saturating_sub(1);
        let mut start = self.cursor;
        let mut left = String::new();
        while start > 0 {
            let ch = self.text[start - 1];
            let candidate = format!("{ch}{left}");
            if Line::raw(candidate.as_str()).width() > width {
                break;
            }
            left = candidate;
            start -= 1;
        }
        let cursor = Line::raw(left.as_str()).width() as u16;
        for ch in self.text[self.cursor..].iter().copied() {
            left.push(ch);
            if Line::raw(left.as_str()).width() > width + 1 {
                left.pop();
                break;
            }
        }
        (left, cursor)
    }
}

impl Field {
    pub fn edits_command(self) -> bool {
        matches!(self, Self::BaseFilter | Self::OutputFields | Self::Filter | Self::Endpoint(_) | Self::Stage(_)
            | Self::Group | Self::TcpPort | Self::Limit)
    }

    pub fn apply_live(self, settings: &mut Refinements, action: &Action, text: &str) -> Result<(), &'static str> {
        let text = text.trim();
        match self {
            Self::BaseFilter => {
                let original = match &action.operation {
                    crate::actions::Operation::Fields { filter, .. } => filter.as_str(),
                    _ => "",
                };
                settings.override_filter = (!text.is_empty() && text != original).then(|| text.to_owned());
            }
            Self::OutputFields => {
                let fields = if text.is_empty() { None } else {
                    let fields: Vec<_> = text.split(',').map(str::trim).map(str::to_owned).collect();
                    if fields.len() > 24 || !fields.iter().all(|field| {
                        !field.is_empty() && field.len() <= 100 && field.as_bytes()[0].is_ascii_alphabetic()
                            && field.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
                    }) {
                        return Err("editor.invalid_fields");
                    }
                    if matches!(action.id.as_str(), "2.2" | "4.6") && !fields.starts_with(action.fields()) {
                        return Err("editor.required_fields");
                    }
                    (fields != action.fields()).then_some(fields)
                };
                settings.output_fields = fields;
                if settings.group.as_ref().is_some_and(|group| {
                    !group.split(',').all(|name| action.editor_fields(settings).iter().any(|field| field == name))
                }) {
                    settings.group = None;
                }
            }
            Self::Filter => settings.filter = text.to_owned(),
            Self::Endpoint(index) => {
                settings.endpoints[index] = if text.is_empty() { None }
                else { Some(text.parse().map_err(|_| "editor.invalid_address")?) };
            }
            Self::TcpPort => {
                settings.tcp_port = if text.is_empty() { None } else {
                    Some(text.parse::<u16>().ok().filter(|n| *n > 0).ok_or("refine.invalid_port")?)
                };
            }
            Self::Limit => {
                settings.limit = if text.is_empty() { None } else {
                    Some(text.parse::<usize>().ok().filter(|n| *n > 0).ok_or("refine.invalid_limit")?)
                };
            }
            Self::Group => {
                if text.is_empty() {
                    settings.group = None;
                } else {
                    let columns: Vec<_> = text.split(',').map(str::trim).collect();
                    if !columns.iter().all(|column| action.editor_fields(settings).iter().any(|field| field == column)) {
                        return Err("refine.invalid_group");
                    }
                    settings.group = Some(columns.join(","));
                    settings.mode = crate::tshark::OutputMode::Count;
                    settings.decode_payload = false;
                }
            }
            Self::Stage(index) => {
                let stage = Stage::parse(text, action).ok_or("pipeline.invalid_stage")?;
                if settings.steps.is_none() {
                    settings.steps = Some(crate::tshark::pipeline::defaults(action, settings));
                }
                let steps = settings.steps.as_mut().expect("pipeline is present");
                steps.get_mut(index).ok_or("pipeline.invalid_stage")?.stage = stage;
            }
            _ => {}
        }
        Ok(())
    }
}
pub fn is_backspace(code: KeyCode, control: bool) -> bool {
    matches!(code, KeyCode::Backspace | KeyCode::Char('\x08' | '\x7f'))
        || (control && code == KeyCode::Char('h'))
}

pub fn export_key(key: char) -> Option<ExportFormat> {
    match key {
        '1' => Some(ExportFormat::Text),
        '2' => Some(ExportFormat::Json),
        '3' => Some(ExportFormat::Csv),
        '4' => Some(ExportFormat::Markdown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backspace_accepts_terminal_encodings_and_respects_cursor() {
        let mut editor = Editor::new("caféxx".into());
        editor.edit(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        editor.edit(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL));
        editor.edit(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        editor.edit(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));
        editor.insert("e");
        assert_eq!(editor.into_text(), "cafe");
    }
}
