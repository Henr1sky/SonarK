use std::{collections::BTreeMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Action {
    pub id: String,
    pub en: String,
    pub pt: String,
    pub log_query: Option<crate::log::Query>,
    #[serde(flatten)]
    pub operation: Operation,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Operation {
    Fields {
        #[serde(default)]
        filter: String,
        fields: Vec<String>,
        #[serde(default)]
        mode: crate::tshark::OutputMode,
        #[serde(default)]
        transform: Option<FieldTransform>,
    },
    Statistics {
        taps: Vec<String>,
    },
    CaptureInfo {
        #[serde(default)]
        flags: Vec<String>,
    },
    Document { tool: DocumentTool },
    Follow { protocol: String },
    PayloadText { format: PayloadTextFormat },
    Objects { protocol: String },
    WriteCapture,
    LogCapture { #[serde(default)] raw: bool },
    Packets {
        format: PacketFormat,
    },
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PacketFormat {
    Hex,
    Json,
    Csv,
    Pdml,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentTool {
    Text,
    Info,
    Images,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayloadTextFormat {
    Decoded,
    Printable,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldTransform {
    MeanByKey,
    SumByKey,
    ConflictingValues,
    MacKind,
    TimeBetween,
    UniquePerWindow,
    EndpointAddress,
}

impl PacketFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Hex => "txt",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Pdml => "xml",
        }
    }
}

impl Action {
    pub fn name(&self, language: crate::i18n::Language) -> &str {
        match language {
            crate::i18n::Language::En => &self.en,
            crate::i18n::Language::Pt => &self.pt,
        }
    }

    pub fn category(&self) -> u16 {
        self.id
            .split_once('.')
            .and_then(|(category, _)| category.parse().ok())
            .expect("Action ID starts with a category number")
    }

    pub fn accepts_filter(&self) -> bool {
        match &self.operation {
            Operation::Fields { .. } | Operation::Packets { .. } | Operation::WriteCapture | Operation::PayloadText { .. } => true,
            Operation::Statistics { taps } => taps.iter().all(|tap| tap != "expert"),
            Operation::CaptureInfo { .. } | Operation::Objects { .. } | Operation::Follow { .. } | Operation::Document { .. } | Operation::LogCapture { .. } => false,
        }
    }

    pub fn is_document(&self) -> bool {
        matches!(self.operation, Operation::Document { .. })
    }

    pub fn is_log_capture(&self) -> bool {
        matches!(self.operation, Operation::LogCapture { .. })
    }

    pub fn has_editor(&self) -> bool {
        matches!(self.id.as_str(), "2.2" | "3.7" | "4.6")
    }

    pub fn fields(&self) -> &[String] {
        match &self.operation {
            Operation::Fields { fields, transform: None, .. } => fields,
            _ => &[],
        }
    }

    pub fn editor_fields<'a>(&'a self, settings: &'a crate::tshark::Refinements) -> &'a [String] {
        if self.has_editor() {
            settings.output_fields.as_deref().unwrap_or_else(|| self.fields())
        } else {
            self.fields()
        }
    }

    pub fn payload_field(&self) -> Option<usize> {
        self.fields().iter().position(|field| matches!(field.as_str(), "tcp.payload" | "data.data"))
    }

    pub fn accepts_tcp_options(&self) -> bool {
        self.fields().iter().any(|field| field.starts_with("tcp."))
            || matches!(&self.operation, Operation::Fields { filter, .. } if filter.contains("tcp."))
            || matches!(self.operation, Operation::PayloadText { .. })
    }

    pub fn writes_files(&self) -> bool {
        matches!(self.operation, Operation::Objects { .. } | Operation::WriteCapture | Operation::LogCapture { .. })
    }

    pub fn default_mode(&self) -> crate::tshark::OutputMode {
        match self.operation {
            Operation::Fields { mode, .. } => mode,
            _ => Default::default(),
        }
    }
}

pub static CATALOG: LazyLock<Vec<Action>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("catalog.json")).expect("Invalid action catalog")
});

pub fn in_category(category: u16) -> impl Iterator<Item = (usize, &'static Action)> {
    CATALOG
        .iter()
        .enumerate()
        .filter(move |(_, action)| action.category() == category)
}


static REQUESTED: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("requested.json")).expect("Invalid requested action inventory")
});

pub fn pending(category: u16) -> Vec<(&'static str, &'static str)> {
    let prefix = format!("{category}.");
    let mut pending: Vec<_> = REQUESTED.iter()
        .filter(|(id, _)| id.starts_with(&prefix) && !CATALOG.iter().any(|action| action.id == **id))
        .map(|(id, title)| (id.as_str(), title.as_str()))
        .collect();
    pending.sort_by_key(|(id, _)| id.split_once('.').and_then(|(_, rank)| rank.parse::<u16>().ok()));
    pending
}
