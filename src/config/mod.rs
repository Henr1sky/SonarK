use std::{
    env, fs,
    io,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::i18n::Language;

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub language: Language,
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        let directory = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| {
                env::var_os("HOME")
                    .map(PathBuf::from)
                    .filter(|path| path.is_absolute())
                    .map(|home| home.join(".config"))
            })?;
        Some(directory.join("sonark/config.json"))
    }

    pub fn load() -> io::Result<Self> {
        let Some(path) = Self::path() else {
            return Ok(Self::default());
        };

        match fs::read(path) {
            Ok(data) => Ok(serde_json::from_slice(&data)?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "XDG_CONFIG_HOME / HOME")
        })?;
        crate::project::write_json(&path, self)
    }
}
