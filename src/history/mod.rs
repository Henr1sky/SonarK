use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Entry {
    pub action: String,
    pub capture: PathBuf,
    pub timestamp_ms: u128,
    pub command: String,
    pub output: String,
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_capture: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_format: Option<crate::actions::PacketFormat>,
}
