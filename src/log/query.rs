use std::io;

use serde::Deserialize;

use crate::{i18n::Language, tshark::OutputMode};
use super::AccessLog;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Query {
    Summary,
    Client,
    HttpVersion,
    Minute,
    Referer,
    UserAgent,
    Uri,
    Method,
    Status,
    QueryString,
    Redirect,
    Timeline,
    Post,
    Connect,
    Errors,
}

impl Query {
    pub fn default_mode(self) -> OutputMode {
        match self {
            Self::Client | Self::HttpVersion | Self::Minute | Self::Method | Self::Status
            | Self::Referer | Self::UserAgent => OutputMode::Count,
            _ => OutputMode::Raw,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Summary => "log_query.summary",
            Self::Client => "log_query.client",
            Self::HttpVersion => "log_query.version",
            Self::Minute => "log_query.minute",
            Self::Referer => "log_query.referer",
            Self::UserAgent => "log_query.agent",
            Self::Uri => "log_query.uri",
            Self::Method => "log_query.method",
            Self::Status => "log_query.status",
            Self::QueryString => "log_query.query",
            Self::Redirect => "log_query.redirect",
            Self::Timeline => "log_query.timeline",
            Self::Post => "log_query.post",
            Self::Connect => "log_query.connect",
            Self::Errors => "log_query.errors",
        }
    }

    pub fn run(self, log: &AccessLog, filter: &str, language: Language) -> io::Result<String> {
        let filter = filter.trim().to_lowercase();
        let mut output = String::new();
        let mut parsed = 0usize;
        let mut selected = 0usize;
        let mut skipped = 0usize;
        let mut bytes = 0u64;
        for (index, line) in log.lines.iter().enumerate() {
            let Some(record) = Record::parse(line) else {
                skipped += 1;
                continue;
            };
            parsed += 1;
            if !filter.is_empty() && !line.to_lowercase().contains(&filter) {
                continue;
            }
            selected += 1;
            if matches!(self, Self::Summary) {
                bytes = bytes.saturating_add(record.bytes);
            } else if let Some(value) = self.project(index + 1, &record) {
                output.push_str(&value);
                output.push('\n');
            }
        }
        if parsed == 0 && !log.lines.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, language.text("log_query.no_records")));
        }
        if matches!(self, Self::Summary) {
            for (key, value) in [
                ("log_query.parsed", parsed.to_string()),
                ("log_query.matched", selected.to_string()),
                ("log_query.skipped", skipped.to_string()),
                ("log_query.bytes", bytes.to_string()),
            ] {
                output.push_str(&format!("{}\t{value}\n", language.text(key)));
            }
        }
        Ok(output)
    }

    fn project(self, line: usize, record: &Record<'_>) -> Option<String> {
        let (method, uri, version) = record.request_triplet();
        match self {
            Self::Summary => None,
            Self::Client => Some(record.client.into()),
            Self::HttpVersion => version.map(str::to_owned),
            Self::Minute => {
                let (time, zone) = record.timestamp.split_once(' ')?;
                let (minute, _) = time.rsplit_once(':')?;
                Some(format!("{minute} {zone}"))
            }
            Self::Referer => recorded(record.referer),
            Self::UserAgent => recorded(record.user_agent),
            Self::Uri => uri.map(str::to_owned),
            Self::Method => method.map(str::to_owned),
            Self::Status => Some(record.status.into()),
            Self::QueryString => uri.and_then(|uri| uri.split_once('?')).map(|(_, query)| query.to_owned()),
            Self::Redirect if record.status.starts_with('3') => Some(record.timeline(line)),
            Self::Post if method == Some("POST") => Some(record.timeline(line)),
            Self::Connect if method == Some("CONNECT") => Some(record.timeline(line)),
            Self::Errors if record.status.starts_with(['4', '5']) => Some(record.timeline(line)),
            Self::Timeline => Some(record.timeline(line)),
            _ => None,
        }
    }
}

fn recorded(field: Option<&str>) -> Option<String> {
    field.filter(|value| !value.is_empty() && *value != "-").map(str::to_owned)
}

pub(super) struct Record<'a> {
    pub(super) client: &'a str,
    pub(super) timestamp: &'a str,
    pub(super) request: &'a str,
    pub(super) status: &'a str,
    pub(super) bytes: u64,
    pub(super) referer: Option<&'a str>,
    pub(super) user_agent: Option<&'a str>,
}

impl<'a> Record<'a> {
    pub(super) fn parse(line: &'a str) -> Option<Self> {
        let (prefix, time) = line.split_once('[')?;
        let client = prefix.split_whitespace().next()?;
        let (timestamp, suffix) = time.split_once(']')?;
        let (request, suffix) = quoted(suffix)?;
        let (status, suffix) = token(suffix)?;
        if status.len() != 3 || !matches!(status.parse::<u16>(), Ok(100..=599)) {
            return None;
        }
        let (bytes, suffix) = token(suffix)?;
        let bytes = if bytes == "-" { 0 } else { bytes.parse().ok()? };
        let (referer, user_agent) = match quoted(suffix) {
            Some((referer, rest)) => (Some(referer), quoted(rest).map(|(value, _)| value)),
            None => (None, None),
        };
        Some(Self { client, timestamp, request, status, bytes, referer, user_agent })
    }

    pub(super) fn request_triplet(&self) -> (Option<&str>, Option<&str>, Option<&str>) {
        let mut fields = self.request.split_whitespace();
        let method = fields.next().filter(|method| *method != "-");
        let uri = fields.next();
        let version = fields.next().filter(|version| version.starts_with("HTTP/"));
        (method, uri, version)
    }

    fn timeline(&self, line: usize) -> String {
        format!("{line}\t{}\t{}\t{}\t{}", self.timestamp, self.client, self.status, self.request)
    }
}

fn token(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start();
    (!text.is_empty()).then(|| text.split_once(char::is_whitespace).unwrap_or((text, "")))
}

fn quoted(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start().strip_prefix('"')?;
    let mut escaped = false;
    for (index, byte) in text.bytes().enumerate() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Some((&text[..index], &text[index + 1..]));
        }
    }
    None
}

pub(super) fn status(line: &str) -> Option<&str> {
    Some(Record::parse(line)?.status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_escaped_quotes_and_combined_fields() {
        let row = r#"192.0.2.2 - - [01/Jan/2026:00:00:00 +0000] "GET /find?q=\"word\" HTTP/1.1" 404 53 "https://example.test" "Agent \"quoted\"""#;
        let record = Record::parse(row).unwrap();
        assert_eq!(record.status, "404");
        assert_eq!(record.referer, Some("https://example.test"));
        assert_eq!(Query::QueryString.project(1, &record).as_deref(), Some(r#"q=\"word\""#));
    }
}
