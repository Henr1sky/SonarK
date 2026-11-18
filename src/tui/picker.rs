use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy)]
pub enum Source {
    Input,
    Reference,
}

impl Source {
    fn preferred(self, path: &Path) -> bool {
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_ascii_lowercase();
        let suffixes: &[&str] = match self {
            Self::Input => &[".pcap", ".pcapng", ".cap", ".pcap.gz", ".pcapng.gz", ".log", ".txt", "_log"],
            Self::Reference => &[".txt", ".log", ".md", ".csv", ".tsv", ".json", ".xml", ".yaml", ".yml", ".pdf"],
        };
        suffixes.iter().any(|suffix| name.ends_with(suffix))
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Input => "picker.input",
            Self::Reference => "picker.reference",
        }
    }
}

pub struct FileEntry {
    pub path: PathBuf,
    pub directory: bool,
}

pub struct FilePicker {
    pub source: Source,
    pub directory: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub show_all: bool,
}

impl FilePicker {
    pub fn new(source: Source, directory: PathBuf) -> io::Result<Self> {
        let mut picker = Self {
            source,
            directory: directory.clone(),
            entries: Vec::new(),
            selected: 0,
            show_all: false,
        };
        picker.open(directory)?;
        Ok(picker)
    }

    pub fn open(&mut self, directory: PathBuf) -> io::Result<()> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            let Ok(metadata) = entry.metadata().and_then(|metadata| {
                if metadata.is_symlink() { path.metadata() } else { Ok(metadata) }
            }) else { continue };
            if metadata.is_dir() || (metadata.is_file()) {
                let hidden = entry.file_name().as_encoded_bytes().starts_with(b".");
                if self.show_all || !hidden {
                    entries.push(FileEntry { path, directory: metadata.is_dir() });
                }
            }
        }
        let order = |entry: &FileEntry| {
            if !entry.directory && self.source.preferred(&entry.path) {
                0
            } else if entry.directory {
                1
            } else {
                2
            }
        };
        entries.sort_by(|a, b| order(a).cmp(&order(b)).then_with(|| a.path.cmp(&b.path)));
        self.directory = directory;
        self.entries = entries;
        self.selected = 0;
        Ok(())
    }

    pub fn parent(&mut self) -> io::Result<()> {
        if let Some(parent) = self.directory.parent() {
            let previous = self.directory.clone();
            self.open(parent.to_owned())?;
            if let Some(index) = self.entries.iter().position(|entry| entry.path == previous) {
                self.selected = index;
            }
        }
        Ok(())
    }

    pub fn toggle_all(&mut self) -> io::Result<()> {
        self.show_all = !self.show_all;
        if let Err(error) = self.open(self.directory.clone()) {
            self.show_all = !self.show_all;
            return Err(error);
        }
        Ok(())
    }
}

pub struct ProjectPicker {
    pub names: Vec<String>,
    pub selected: usize,
}

pub fn expand_home(text: &str) -> PathBuf {
    if let Some(rest) = text.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(text)
}
