use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Pt,
}

impl Language {
    pub fn other(self) -> Self {
        match self {
            Self::En => Self::Pt,
            Self::Pt => Self::En,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Pt => "Português",
        }
    }

    pub fn text(self, key: &'static str) -> &'static str {
        let localized = match self {
            Self::En => None,
            Self::Pt => PT.get(key),
        };
        localized.or_else(|| EN.get(key)).map_or(key, String::as_str)
    }
}

static EN: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("en.json")).expect("Invalid English catalog")
});

static PT: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("pt.json")).expect("Invalid Portuguese catalog")
});
