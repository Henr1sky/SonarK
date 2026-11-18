pub mod pipeline;
mod columns;
pub use columns::transform;

use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    io::{self, Read},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use serde::{Deserialize, Serialize};

use crate::actions::{Action, DocumentTool, Operation, PacketFormat, PayloadTextFormat};

const OUTPUT_BYTES: usize = 2 * 1024 * 1024;
const ERROR_BYTES: usize = 32 * 1024;

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    #[default]
    Raw,
    Unique,
    Count,
}

impl OutputMode {
    pub fn next(self) -> Self {
        match self {
            Self::Raw => Self::Unique,
            Self::Unique => Self::Count,
            Self::Count => Self::Raw,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Raw => "refine.raw",
            Self::Unique => "refine.unique",
            Self::Count => "refine.count",
        }
    }
}

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Both,
    ToPort,
    FromPort,
}

impl Direction {
    pub fn next(self) -> Self {
        match self {
            Self::Both => Self::ToPort,
            Self::ToPort => Self::FromPort,
            Self::FromPort => Self::Both,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Both => "refine.direction_both",
            Self::ToPort => "refine.direction_to",
            Self::FromPort => "refine.direction_from",
        }
    }

    fn port_field(self) -> &'static str {
        match self {
            Self::Both => "tcp.port",
            Self::ToPort => "tcp.dstport",
            Self::FromPort => "tcp.srcport",
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Refinements {
    pub filter: String,
    pub limit: Option<usize>,
    pub mode: OutputMode,
    pub stream: u32,
    pub tcp_port: Option<u16>,
    pub direction: Direction,
    pub psh_only: bool,
    pub decode_payload: bool,
    pub group: Option<String>,
    pub endpoints: [Option<std::net::IpAddr>; 2],
    pub override_filter: Option<String>,
    pub output_fields: Option<Vec<String>>,
    pub steps: Option<Vec<pipeline::Step>>,
}

pub struct Invocation {
    pub program: &'static str,
    pub args: Vec<OsString>,
}

impl Invocation {
    pub fn new(action: &Action, capture: &Path, settings: &Refinements) -> Self {
        Self::with_destination(action, capture, settings, Path::new("<export-directory>"))
    }

    pub fn with_destination(action: &Action, capture: &Path, settings: &Refinements, destination: &Path) -> Self {
        let capture = capture.as_os_str().to_owned();
        match &action.operation {
            Operation::CaptureInfo { flags } => {
                let mut args: Vec<OsString> = flags.iter().map(OsString::from).collect();
                args.extend(["--".into(), capture]);
                return Self { program: "capinfos", args };
            }
            Operation::Document { tool } => {
                return match tool {
                    DocumentTool::Text => Self {
                        program: "pdftotext", args: vec![capture, "-".into()],
                    },
                    DocumentTool::Info => Self {
                        program: "pdfinfo", args: vec![capture],
                    },
                    DocumentTool::Images => Self {
                        program: "pdfimages", args: vec!["-list".into(), capture],
                    },
                };
            }
            _ => {}
        }

        let mut args = vec!["-n".into(), "-r".into(), capture];
        if action.has_editor() {
            args.push("-l".into());
        }
        let mut clauses = Vec::new();
        let extra = settings.filter.trim();
        if !extra.is_empty() {
            clauses.push(extra.to_owned());
        }
        if action.has_editor() {
            for (direction, endpoint) in ["src", "dst"].into_iter().zip(settings.endpoints) {
                if let Some(endpoint) = endpoint {
                    let field = if endpoint.is_ipv4() { "ip" } else { "ipv6" };
                    clauses.push(format!("{field}.{direction} == {endpoint}"));
                }
            }
        }
        if action.accepts_tcp_options() {
            if let Some(port) = settings.tcp_port {
                clauses.push(format!("{} == {port}", settings.direction.port_field()));
            }
            if settings.psh_only {
                clauses.push("tcp.flags.push == 1".to_owned());
            }
        }
        let filter = if clauses.len() <= 1 {
            clauses.pop().unwrap_or_default()
        } else {
            clauses.iter().map(|clause| format!("({clause})")).collect::<Vec<_>>().join(" && ")
        };
        let filter = filter.as_str();
        match &action.operation {
            Operation::Fields {
                filter: base,
                fields,
                transform,
                ..
            } => {
                let base = settings.override_filter.as_deref().filter(|_| action.has_editor())
                    .unwrap_or(base);
                let combined = match (base.is_empty(), filter.is_empty()) {
                    (true, _) => filter.to_owned(),
                    (_, true) => base.to_owned(),
                    _ => format!("({base}) && ({filter})"),
                };
                if !combined.is_empty() {
                    args.extend(["-Y".into(), combined.into()]);
                }
                args.extend(["-T".into(), "fields".into()]);
                if transform.is_some() || action.has_editor() || action.payload_field().is_some() {
                    args.extend(["-E".into(), "occurrence=f".into()]);
                }
                let fields = if action.has_editor() { action.editor_fields(settings) } else { fields };
                for field in fields {
                    args.extend(["-e".into(), field.into()]);
                }
            }
            Operation::Statistics { taps } => {
                args.push("-q".into());
                for tap in taps {
                    let value = if filter.is_empty() || !action.accepts_filter() {
                        tap.clone()
                    } else {
                        format!("{tap},{filter}")
                    };
                    args.extend(["-z".into(), value.into()]);
                }
            }
            Operation::Packets { format } => {
                if !filter.is_empty() {
                    args.extend(["-Y".into(), filter.into()]);
                }
                let options: &[&str] = match format {
                    PacketFormat::Hex => &["-x"],
                    PacketFormat::Json => &["-T", "json"],
                    PacketFormat::Csv => &[
                        "-T", "fields", "-E", "header=y", "-E", "separator=,",
                        "-E", "quote=d", "-e", "frame.number", "-e", "frame.time_epoch",
                        "-e", "frame.protocols", "-e", "frame.len",
                    ],
                    PacketFormat::Pdml => &["-T", "pdml"],
                };
                args.extend(options.iter().map(OsString::from));
            }
            Operation::Follow { protocol } => {
                args.extend(["-q".into(), "-z".into(),
                    format!("follow,{protocol},ascii,{}", settings.stream).into()]);
            }
            Operation::PayloadText { .. } => {
                let display = if filter.is_empty() {
                    "tcp.payload".to_owned()
                } else {
                    format!("tcp.payload && ({filter})")
                };
                args.extend(["-Y".into(), display.into(), "-T".into(), "fields".into(),
                    "-E".into(), "occurrence=f".into(), "-e".into(), "tcp.payload".into()]);
            }
            Operation::Objects { protocol } => {
                let mut argument = OsString::from(format!("{protocol},"));
                argument.push(destination);
                args.extend(["-q".into(), "--export-objects".into(), argument]);
            }
            Operation::WriteCapture => {
                if !filter.is_empty() {
                    args.extend(["-Y".into(), filter.into()]);
                }
                args.extend(["-w".into(), destination.join("filtered.pcapng").into_os_string()]);
            }
            Operation::CaptureInfo { .. } | Operation::Document { .. } | Operation::LogCapture { .. } => unreachable!(),
        }
        Self {
            program: "tshark",
            args,
        }
    }

    pub fn import_log(format: crate::log::capture::Format, directory: &Path) -> Self {
        use crate::log::capture::{HEX_FILE, OUTPUT};
        let mut args: Vec<OsString> = ["-q", "-F", "pcapng", "-P", format.dissector(), "-N", format.interface()]
            .into_iter().map(OsString::from).collect();
        if format.writes_recorded_time() {
            args.extend(["-t".into(), "ISO".into()]);
        }
        args.extend([
            directory.join(HEX_FILE).into_os_string(),
            directory.join(OUTPUT).into_os_string(),
        ]);
        Self { program: "text2pcap", args }
    }

    pub fn preview(&self) -> String {
        let args = self.args.iter().map(|arg| quote(arg)).collect::<Vec<_>>();
        format!("{} {}", self.program, args.join(" "))
    }

    pub fn start(&self) -> io::Result<Running> {
        let mut child = Command::new(self.program)
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().expect("piped output");
        let stderr = child.stderr.take().expect("piped diagnostics");
        let preview = Arc::new(Mutex::new(Vec::new()));
        let writer = Arc::clone(&preview);
        Ok(Running {
            child,
            preview,
            stdout: Some(thread::spawn(move || collect_output(stdout, OUTPUT_BYTES, Some(&writer)))),
            stderr: Some(thread::spawn(move || read_bounded(stderr, ERROR_BYTES))),
        })
    }
}

fn quote(arg: &OsStr) -> String {
    let text = arg.to_string_lossy();
    if !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-.=/,:".contains(&byte))
    {
        return text.into_owned();
    }
    format!("'{}'", text.replace('\'', "'\\''"))
}

pub struct Running {
    child: Child,
    preview: Arc<Mutex<Vec<u8>>>,
    stdout: Option<JoinHandle<io::Result<Captured>>>,
    stderr: Option<JoinHandle<io::Result<Captured>>>,
}

impl Running {
    pub fn preview(&self) -> String {
        let Ok(buffer) = self.preview.lock() else { return String::new() };
        let end = buffer.iter().rposition(|&byte| byte == b'\n').map_or(0, |i| i + 1);
        sanitize(&String::from_utf8_lossy(&buffer[..end]))
    }

    pub fn poll(&mut self) -> io::Result<Option<Completed>> {
        let Some(status) = self.child.try_wait()? else {
            return Ok(None);
        };
        let stdout = finish(&mut self.stdout)?;
        let stderr = finish(&mut self.stderr)?;
        Ok(Some(Completed {
            status,
            text: String::from_utf8_lossy(&stdout.bytes).into_owned(),
            diagnostics: sanitize(&String::from_utf8_lossy(&stderr.bytes)),
            truncated: stdout.truncated,
        }))
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

struct Captured {
    bytes: Vec<u8>,
    truncated: bool,
}

fn read_bounded(input: impl Read, limit: usize) -> io::Result<Captured> {
    collect_output(input, limit, None)
}

fn collect_output(mut input: impl Read, limit: usize, preview: Option<&Mutex<Vec<u8>>>) -> io::Result<Captured> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 8192];
    let mut truncated = false;
    loop {
        let size = input.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        let amount = size.min(limit - bytes.len());
        bytes.extend_from_slice(&buffer[..amount]);
        if let Some(preview) = preview {
            if let Ok(mut preview) = preview.lock() {
                let preview_limit = 64 * 1024usize;
                let shown = size.min(preview_limit.saturating_sub(preview.len()));
                preview.extend_from_slice(&buffer[..shown]);
            }
        }
        truncated |= amount < size;
    }
    if truncated {
        let end = bytes.iter().rposition(|byte| *byte == b'\n').map_or(0, |i| i + 1);
        bytes.truncate(end);
    }
    Ok(Captured { bytes, truncated })
}

fn finish(reader: &mut Option<JoinHandle<io::Result<Captured>>>) -> io::Result<Captured> {
    reader
        .take()
        .expect("output read once")
        .join()
        .map_err(|_| io::Error::other("Could not collect process output"))?
}

pub struct Completed {
    pub status: ExitStatus,
    pub text: String,
    pub diagnostics: String,
    pub truncated: bool,
}

fn decode_hex(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut high = None;
    for ch in hex.chars().filter(|&ch| ch != ':') {
        let Some(digit) = ch.to_digit(16) else { return Vec::new() };
        if let Some(first) = high.take() {
            bytes.push((first * 16 + digit) as u8);
        } else {
            high = Some(digit);
        }
    }
    if high.is_none() { bytes } else { Vec::new() }
}

pub fn format_fields(output: &str, action: &Action, settings: &Refinements) -> String {
    let fields = action.fields();
    if let Some(index) = settings.group.as_ref().and_then(|group| fields.iter().position(|field| field == group)) {
        return output.lines().filter_map(|row| row.split('\t').nth(index))
            .filter(|value| !value.is_empty()).collect::<Vec<_>>().join("\n");
    }
    if let Some(payload_index) = action.payload_field().filter(|_| settings.decode_payload) {
        let field_index = |name: &str| fields.iter().position(|field| field == name);
        let source_ip = field_index("ip.src");
        let source_ipv6 = field_index("ipv6.src");
        let source_port = field_index("tcp.srcport");
        let dest_port = field_index("tcp.dstport");
        let mut text = String::new();
        for row in output.lines() {
            let columns: Vec<_> = row.split('\t').collect();
            let value = |index: Option<usize>| index.and_then(|index| columns.get(index).copied())
                .filter(|value| !value.is_empty());
            let Some(hex) = value(Some(payload_index)) else { continue };
            let bytes = decode_hex(hex);
            if bytes.is_empty() { continue; }
            let utf8 = String::from_utf8_lossy(&bytes);
            let source = value(source_ip).or_else(|| value(source_ipv6));
            if fields[payload_index] == "data.data" {
                for ch in utf8.trim_end_matches(['\r', '\n']).chars() {
                    text.push(if ch.is_control() { '·' } else { ch });
                }
                text.push('\n');
            } else {
                let label = format!("{} ({} → {}): ", source.unwrap_or("-"),
                    value(source_port).unwrap_or("-"), value(dest_port).unwrap_or("-"));
                for line in utf8.lines() {
                    text.push_str(&label);
                    text.extend(line.chars().map(|ch| if ch.is_control() { '·' } else { ch }));
                    text.push('\n');
                }
            }
        }
        return text;
    }
    output.to_owned()
}

pub fn payload_text(payload: &str, format: PayloadTextFormat) -> String {
    let mut output = String::new();
    for line in payload.lines() {
        let bytes = decode_hex(line);
        match format {
            PayloadTextFormat::Decoded => {
                for ch in String::from_utf8_lossy(&bytes).chars() {
                    output.push(if ch.is_control() { '·' } else { ch });
                }
                output.push('\n');
            }
            PayloadTextFormat::Printable => {
                for run in bytes.split(|byte| !matches!(byte, 0x20..=0x7e)).filter(|run| run.len() >= 4) {
                    output.push_str(std::str::from_utf8(run).expect("ASCII run"));
                    output.push('\n');
                }
            }
        }
    }
    output
}

pub fn limit_text(mut text: String) -> (String, bool) {
    let truncated = text.len() > OUTPUT_BYTES;
    if truncated {
        let end = text.as_bytes()[..OUTPUT_BYTES].iter().rposition(|&byte| byte == b'\n').unwrap_or(0);
        text.truncate(end);
    }
    (text, truncated)
}

pub fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\t'))
        .collect()
}

pub fn refine(output: &str, settings: &Refinements) -> String {
    let limit = settings.limit.unwrap_or(usize::MAX);
    if settings.mode == OutputMode::Raw {
        return output.lines().take(limit).collect::<Vec<_>>().join("\n");
    }
    let mut counts = BTreeMap::new();
    for row in output.lines().filter(|line| !line.trim().is_empty()) {
        *counts.entry(row).or_insert(0usize) += 1;
    }
    if settings.mode == OutputMode::Unique {
        return counts.into_keys().take(limit).collect::<Vec<_>>().join("\n");
    }
    let mut rows: Vec<_> = counts.into_iter().collect();
    rows.sort_by(|(a, a_count), (b, b_count)| b_count.cmp(a_count).then_with(|| a.cmp(b)));
    rows.into_iter()
        .take(limit)
        .map(|(row, count)| format!("{count}\t{row}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_path_and_combined_filter_are_single_arguments() {
        let action = crate::actions::CATALOG.iter().find(|action| action.id == "10.1").unwrap();
        let path = Path::new("/tmp/operator's capture.pcapng");
        let settings = Refinements {
            filter: "udp.port == 53 || tcp.port == 53".into(),
            ..Default::default()
        };
        let invocation = Invocation::new(action, path, &settings);
        assert_eq!(invocation.args[2], path.as_os_str());
        let filter = invocation.args.iter().position(|arg| arg == "-Y").unwrap();
        assert_eq!(
            invocation.args[filter + 1],
            "(dns.flags.response == 0 && dns.qry.name) && (udp.port == 53 || tcp.port == 53)"
        );
    }

    #[test]
    fn directed_tcp_filter_and_text_output_keep_the_ports() {
        let action = crate::actions::CATALOG.iter().find(|action| action.id == "4.6").unwrap();
        let settings = Refinements {
            tcp_port: Some(8089),
            direction: Direction::ToPort,
            psh_only: true,
            decode_payload: true,
            ..Default::default()
        };
        let invocation = Invocation::new(action, Path::new("/tmp/example.pcap"), &settings);
        assert!(invocation.args.iter().any(|arg| {
            arg == "(tcp.payload) && ((tcp.dstport == 8089) && (tcp.flags.push == 1))"
        }));
        assert_eq!(
            format_fields("192.0.2.4\t\t50790\t8089\t68656c6c6f\n", action, &settings),
            "192.0.2.4 (50790 → 8089): hello\n"
        );
    }

    #[test]
    fn frequency_is_sorted_before_applying_limit() {
        let settings = Refinements {
            mode: OutputMode::Count,
            limit: Some(2),
            ..Default::default()
        };
        assert_eq!(refine("z\nb\na\nz\n", &settings), "2\tz\n1\ta");
    }

    #[test]
    fn output_cap_retains_only_complete_lines() {
        let captured = read_bounded("first\nsecond\n".as_bytes(), 8).unwrap();
        assert!(captured.truncated);
        assert_eq!(captured.bytes, b"first\n");
    }
}
