use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{history::Entry, tshark::Refinements};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Data {
    pub capture: Option<PathBuf>,
    pub access_log: Option<PathBuf>,
    pub derived_capture: Option<PathBuf>,
    pub use_derived: bool,
    pub references: Vec<PathBuf>,
    pub notes: String,
    pub presets: BTreeMap<String, Refinements>,
    pub favorites: BTreeSet<String>,
    pub history: Vec<Entry>,
}

pub struct Project {
    pub name: String,
    pub data: Data,
    directory: PathBuf,
    _lock: fs::File,
}

impl Project {
    pub fn open(name: String) -> io::Result<Self> {
        validate_name(&name)?;
        let directory = projects_directory()?.join(&name);
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&directory)?;
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(directory.join(".lock"))?;
        lock.try_lock().map_err(io::Error::from)?;

        let data = match fs::read(directory.join("project.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Data::default(),
            Err(error) => return Err(error),
        };
        Ok(Self {
            name,
            data,
            directory,
            _lock: lock,
        })
    }

    pub fn create(name: String, data: Data) -> io::Result<Self> {
        validate_name(&name)?;
        let directory = projects_directory()?.join(&name);
        fs::DirBuilder::new().mode(0o700).create(&directory)?;
        let mut project = Self::open(name)?;
        project.data = data;
        if let Err(error) = project.save() {
            let _ = fs::remove_file(directory.join(".lock"));
            let _ = fs::remove_dir(directory);
            return Err(error);
        }
        Ok(project)
    }

    pub fn save(&self) -> io::Result<()> {
        write_json(&self.directory.join("project.json"), &self.data)
    }

    pub fn names() -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::read_dir(projects_directory()?)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Ok(name) = entry.file_name().into_string() {
                    if validate_name(&name).is_ok() {
                        names.push(name);
                    }
                }
            }
        }
        names.sort();
        Ok(names)
    }

    pub fn rename(&mut self, name: &str) -> io::Result<()> {
        validate_name(name)?;
        if name == self.name {
            return Ok(());
        }
        let destination = self.directory.with_file_name(name);
        if destination.try_exists()? {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Project already exists"));
        }
        fs::rename(&self.directory, &destination)?;
        self.directory = destination;
        self.name = name.to_owned();
        Ok(())
    }

    pub fn set_input(&mut self, input: InputFile) -> io::Result<()> {
        let old_capture = self.data.capture.take();
        let old_log = self.data.access_log.take();
        let old_derived = self.data.derived_capture.take();
        let was_derived = std::mem::take(&mut self.data.use_derived);
        match input {
            InputFile::Pcap(path) => self.data.capture = Some(path),
            InputFile::Log(path) => self.data.access_log = Some(path),
        }
        if let Err(error) = self.save() {
            self.data.capture = old_capture;
            self.data.access_log = old_log;
            self.data.derived_capture = old_derived;
            self.data.use_derived = was_derived;
            return Err(error);
        }
        Ok(())
    }

    pub fn activate_derived(&mut self, source: &Path, path: &Path) -> io::Result<()> {
        if self.data.access_log.as_deref() != Some(source) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Derived capture belongs to a different log"));
        }
        let path = readable_file(path)?;
        let previous = self.data.derived_capture.replace(path);
        let was_derived = self.data.use_derived;
        self.data.use_derived = true;
        if let Err(error) = self.save() {
            self.data.derived_capture = previous;
            self.data.use_derived = was_derived;
            return Err(error);
        }
        Ok(())
    }

    pub fn toggle_derived(&mut self) -> io::Result<bool> {
        if let (Some(_), Some(path)) = (&self.data.access_log, &self.data.derived_capture) {
            if !self.data.use_derived {
                readable_file(path)?;
            }
            self.data.use_derived = !self.data.use_derived;
            if let Err(error) = self.save() {
                self.data.use_derived = !self.data.use_derived;
                return Err(error);
            }
            Ok(self.data.use_derived)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Generate and activate a derived capture first"))
        }
    }

    pub fn record(&mut self, entry: Entry) -> io::Result<()> {
        const HISTORY_LIMIT: usize = 20;
        self.data.history.push(entry);
        let removed = if self.data.history.len() > HISTORY_LIMIT {
            Some(self.data.history.remove(0))
        } else {
            None
        };
        if let Err(error) = self.save() {
            self.data.history.pop();
            if let Some(removed) = removed {
                self.data.history.insert(0, removed);
            }
            return Err(error);
        }
        Ok(())
    }

    pub fn artifact_directory(&self, action: &str) -> io::Result<ArtifactDirectory> {
        let (category, _) = action.split_once('.').ok_or_else(invalid_action)?;
        let parent = self.directory.join("exports").join(category);
        fs::DirBuilder::new().recursive(true).mode(0o700).create(&parent)?;
        let path = parent.join(format!("{action}_{}", timestamp()));
        fs::DirBuilder::new().mode(0o700).create(&path)?;
        Ok(ArtifactDirectory { path, kept: false })
    }

    pub fn export(&self, entry: &Entry, format: ExportFormat) -> io::Result<PathBuf> {
        let (category, _) = entry.action.split_once('.').ok_or_else(invalid_action)?;
        if !entry.action.bytes().all(|byte| byte.is_ascii_digit() || byte == b'.') {
            return Err(invalid_action());
        }
        let directory = self.directory.join("exports").join(category);
        fs::create_dir_all(&directory)?;
        let extension = match format {
            ExportFormat::Native => entry.native_format.map(|format| format.extension())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Native packet output is unavailable"))?,
            _ => format.extension(),
        };
        let path = directory.join(format!("{}_{}.{}", entry.action, timestamp(), extension));
        let body = match format {
            ExportFormat::Text | ExportFormat::Native => entry.output.clone(),
            ExportFormat::Json => serde_json::to_string_pretty(entry)? + "\n",
            ExportFormat::Csv => {
                let mut csv = String::from("line,text\n");
                for (index, line) in entry.output.lines().enumerate() {
                    let prefix = if line.trim_start().starts_with(['=', '+', '-', '@']) {
                        "'"
                    } else {
                        ""
                    };
                    csv.push_str(&format!(
                        "{},\"{prefix}{}\"\n",
                        index + 1,
                        line.replace('"', "\"\"")
                    ));
                }
                csv
            }
            ExportFormat::Markdown => {
                let mut text = format!("# SonarK [{}]\n\n", entry.action);
                for line in entry.output.lines() {
                    text.push_str(&format!("    {line}\n"));
                }
                text
            }
        };
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        if let Err(error) = file.write_all(body.as_bytes()).and_then(|_| file.sync_all()) {
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        Ok(path)
    }
}

pub struct ArtifactDirectory {
    pub path: PathBuf,
    kept: bool,
}

impl ArtifactDirectory {
    pub fn finish(mut self) -> io::Result<String> {
        let mut files = fs::read_dir(&self.path)?
            .map(|entry| entry.map(|entry| entry.path())).collect::<io::Result<Vec<_>>>()?;
        files.sort();
        let mut report = format!("{}\n", self.path.display());
        for path in files {
            report.push_str(&format!("{}\n", path.file_name().unwrap_or_default().to_string_lossy()));
        }
        self.kept = true;
        Ok(report)
    }
}

impl Drop for ArtifactDirectory {
    fn drop(&mut self) {
        if !self.kept {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[derive(Clone)]
pub enum InputFile {
    Pcap(PathBuf),
    Log(PathBuf),
}

impl InputFile {
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let path = readable_file(path)?;
        let mut file = fs::File::open(&path)?;
        let mut header = [0; 4];
        let size = file.read(&mut header)?;
        let has_capture_header = size == 4 && matches!(
            header,
            [0xd4, 0xc3, 0xb2, 0xa1] | [0xa1, 0xb2, 0xc3, 0xd4]
                | [0x4d, 0x3c, 0xb2, 0xa1] | [0xa1, 0xb2, 0x3c, 0x4d]
                | [0x0a, 0x0d, 0x0d, 0x0a]
        );
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_ascii_lowercase();
        if has_capture_header || [".pcap", ".pcapng", ".cap", ".pcap.gz", ".pcapng.gz"]
            .iter().any(|suffix| name.ends_with(suffix))
        {
            Ok(Self::Pcap(path))
        } else {
            crate::log::AccessLog::open(&path)?;
            Ok(Self::Log(path))
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Pcap(path) | Self::Log(path) => path,
        }
    }
}

impl Data {
    pub fn input_path(&self) -> Option<&Path> {
        self.active_derived().or(self.access_log.as_deref()).or(self.capture.as_deref())
    }

    pub fn active_derived(&self) -> Option<&Path> {
        self.use_derived.then_some(self.derived_capture.as_deref()).flatten()
    }

    pub fn active_log(&self) -> Option<&Path> {
        self.access_log.as_deref().filter(|_| self.active_derived().is_none())
    }
}

pub fn readable_file(path: &Path) -> io::Result<PathBuf> {
    let path = path.canonicalize()?;
    if !path.metadata()?.is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Expected a regular file"));
    }
    fs::File::open(&path)?;
    Ok(path)
}

fn validate_name(name: &str) -> io::Result<()> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Expected a project name of 1–64 ASCII letters, digits, hyphens or underscores",
        ));
    }
    Ok(())
}

fn projects_directory() -> io::Result<PathBuf> {
    let directory = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .map(|home| home.join(".local/share"))
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "XDG_DATA_HOME / HOME"))?;
    Ok(directory.join("sonark/projects/saves"))
}

fn invalid_action() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "Invalid action ID")
}

pub fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn write_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let directory = path.parent().expect("application data path has a parent");
    fs::DirBuilder::new().recursive(true).mode(0o700).create(directory)?;
    let pending = directory.join(format!(".{}.{}.tmp", std::process::id(), timestamp()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&pending)?;
    let result = (|| {
        serde_json::to_writer_pretty(&mut file, value)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&pending, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(pending);
    }
    result
}

#[derive(Clone, Copy)]
pub enum ExportFormat {
    Text,
    Json,
    Csv,
    Markdown,
    Native,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Markdown => "md",
            Self::Native => unreachable!(),
        }
    }
}
