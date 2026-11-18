pub mod capture;
mod query;
pub use query::Query;

use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read},
    path::Path,
};

const PREVIEW_BYTES: usize = 16 * 1024 * 1024;

pub struct AccessLog {
    lines: Vec<String>,
    pub matches: Vec<usize>,
    pub query: String,
    pub scroll: usize,
    pub horizontal: u16,
    pub truncated: bool,
    pub statuses: BTreeMap<String, usize>,
}

impl AccessLog {
    pub fn open(path: &Path) -> io::Result<Self> {
        let mut bytes = Vec::new();
        File::open(path)?.take((PREVIEW_BYTES + 1) as u64).read_to_end(&mut bytes)?;
        if bytes.contains(&0) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected a text access log"));
        }
        let truncated = bytes.len() > PREVIEW_BYTES;
        if truncated {
            bytes.truncate(bytes[..PREVIEW_BYTES].iter().rposition(|&byte| byte == b'\n').map_or(0, |i| i + 1));
        }
        let lines = String::from_utf8_lossy(&bytes).lines()
            .map(crate::tshark::sanitize).collect();
        let mut log = Self {
            lines,
            matches: Vec::new(),
            query: String::new(),
            scroll: 0,
            horizontal: 0,
            truncated,
            statuses: BTreeMap::new(),
        };
        log.filter(String::new());
        Ok(log)
    }

    pub fn filter(&mut self, query: String) {
        let needle = query.to_lowercase();
        self.matches.clear();
        self.statuses.clear();
        for (index, line) in self.lines.iter().enumerate() {
            if !needle.is_empty() && !line.to_lowercase().contains(&needle) {
                continue;
            }
            self.matches.push(index);
            if let Some(status) = query::status(line) {
                *self.statuses.entry(status.to_owned()).or_default() += 1;
            }
        }
        self.query = query;
        self.scroll = 0;
    }

    pub fn visible(&self, limit: usize) -> impl Iterator<Item = (usize, &str)> {
        self.matches.iter().skip(self.scroll).take(limit)
            .map(|&index| (index + 1, self.lines[index].as_str()))
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_common_status_without_treating_path_as_status() {
        let log = r#"192.0.2.2 - - [01/Jan/2026:00:00:00 +0000] "GET /200 HTTP/1.1" 404 53 "-" "-""#;
        assert_eq!(query::status(log), Some("404"));
        assert_eq!(query::status("application error: 404"), None);
    }
}
