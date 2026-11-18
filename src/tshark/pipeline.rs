use std::{
    io::{self, Write},
    os::unix::process::ExitStatusExt,
    process::{Command, Stdio},
    thread,
};

use serde::{Deserialize, Serialize};

use crate::actions::Action;
use super::{OutputMode, Refinements, quote};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Step {
    pub enabled: bool,
    pub stage: Stage,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Sort { kind: SortKind, reverse: bool, column: Option<usize> },
    Unique,
    Count,
    Head(usize),
    Tail(usize),
    Select(Vec<usize>),
    Search { mode: Search, pattern: String },
    Lines,
    Reduce { mean: bool, key: usize, value: usize },
    Origin,
    Decode,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortKind { Text, Number, Version, Date }

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Search { Literal, IgnoreCase, Regex, Exclude }

pub fn enabled(stage: Stage) -> Step {
    Step { enabled: true, stage }
}

pub fn defaults(action: &Action, settings: &Refinements) -> Vec<Step> {
    let mut stages = Vec::new();
    if let Some(fields) = &settings.group {
        let indexes: Vec<_> = fields.split(',').filter_map(|name| {
            let name = name.trim();
            action.editor_fields(settings).iter().position(|field| field == name).map(|index| index + 1)
        }).collect();
        if !indexes.is_empty() {
            stages.push(Stage::Select(indexes));
        }
    } else if action.id == "2.2" {
        stages.push(Stage::Origin);
    } else if action.id == "4.6" && settings.decode_payload {
        stages.push(Stage::Decode);
    }
    let kind = if action.id == "3.7" && settings.group.is_none() { SortKind::Number } else { SortKind::Text };
    match settings.mode {
        OutputMode::Raw => {}
        OutputMode::Unique => {
            stages.push(Stage::Sort { kind, reverse: false, column: None });
            stages.push(Stage::Unique);
        }
        OutputMode::Count => {
            stages.push(Stage::Sort { kind, reverse: false, column: None });
            stages.push(Stage::Count);
            stages.push(Stage::Sort { kind: SortKind::Number, reverse: true, column: None });
        }
    }
    if let Some(limit) = settings.limit {
        stages.push(Stage::Head(limit));
    }
    stages.into_iter().map(enabled).collect()
}

pub fn bank(action: &Action) -> Vec<Stage> {
    let mut stages = vec![
        Stage::Sort { kind: SortKind::Text, reverse: false, column: None },
        Stage::Sort { kind: SortKind::Number, reverse: true, column: None },
        Stage::Sort { kind: SortKind::Version, reverse: false, column: None },
        Stage::Sort { kind: SortKind::Date, reverse: false, column: None },
        Stage::Unique, Stage::Count, Stage::Head(10), Stage::Tail(10),
        Stage::Search { mode: Search::Literal, pattern: "example".into() },
        Stage::Select(vec![1]), Stage::Lines,
        Stage::Reduce { mean: false, key: 1, value: 2 },
        Stage::Reduce { mean: true, key: 1, value: 2 },
    ];
    if action.id == "2.2" { stages.push(Stage::Origin); }
    if action.id == "4.6" { stages.push(Stage::Decode); }
    stages
}

impl Stage {
    pub fn key(&self) -> &'static str {
        match self {
            Self::Sort { kind: SortKind::Text, .. } => "pipeline.sort_text",
            Self::Sort { kind: SortKind::Number, .. } => "pipeline.sort_number",
            Self::Sort { kind: SortKind::Version, .. } => "pipeline.sort_version",
            Self::Sort { kind: SortKind::Date, .. } => "pipeline.sort_date",
            Self::Unique => "pipeline.unique",
            Self::Count => "pipeline.count",
            Self::Head(_) => "pipeline.head",
            Self::Tail(_) => "pipeline.tail",
            Self::Select(_) => "pipeline.select",
            Self::Search { .. } => "pipeline.search",
            Self::Lines => "pipeline.lines",
            Self::Reduce { mean: false, .. } => "pipeline.sum",
            Self::Reduce { mean: true, .. } => "pipeline.mean",
            Self::Origin => "pipeline.origin",
            Self::Decode => "pipeline.decode",
        }
    }

    pub fn editor_text(&self) -> String {
        match self {
            Self::Sort { kind, reverse, column } => {
                let kind = match kind { SortKind::Text => "text", SortKind::Number => "number", SortKind::Version => "version", SortKind::Date => "date" };
                let order = if *reverse { "desc" } else { "asc" };
                format!("sort {kind} {order}{}", column.map_or_else(String::new, |id| format!(" {id}")))
            }
            Self::Unique => "uniq".into(),
            Self::Count => "uniq-count".into(),
            Self::Head(amount) => format!("head {amount}"),
            Self::Tail(amount) => format!("tail {amount}"),
            Self::Select(columns) => format!("cut {}", columns.iter().map(usize::to_string).collect::<Vec<_>>().join(",")),
            Self::Search { mode, pattern } => {
                let mode = match mode {
                    Search::Literal => "literal", Search::IgnoreCase => "ignore-case",
                    Search::Regex => "regex", Search::Exclude => "exclude",
                };
                format!("grep {mode} {pattern}")
            }
            Self::Lines => "wc".into(),
            Self::Reduce { mean, key, value } => format!("{} {key} {value}", if *mean { "mean" } else { "sum" }),
            Self::Origin => "origin".into(),
            Self::Decode => "decode".into(),
        }
    }

    pub fn parse(text: &str, action: &Action) -> Option<Self> {
        let text = text.trim();
        if let Some(rest) = text.strip_prefix("grep ") {
            let (kind, pattern) = rest.split_once(' ')?;
            let mode = match kind {
                "literal" => Search::Literal, "ignore-case" => Search::IgnoreCase,
                "regex" => Search::Regex, "exclude" => Search::Exclude, _ => return None,
            };
            return (!pattern.trim().is_empty() && !pattern.contains(['\0', '\n', '\r']))
                .then(|| Self::Search { mode, pattern: pattern.trim().to_owned() });
        }
        let parts: Vec<_> = text.split_whitespace().collect();
        let positive = |value: &str| value.parse::<usize>().ok().filter(|value| (1..=1_000_000).contains(value));
        match parts.as_slice() {
            ["sort", kind, order] | ["sort", kind, order, _] => {
                let kind = match *kind {
                    "text" => SortKind::Text, "number" => SortKind::Number, "version" => SortKind::Version, "date" => SortKind::Date, _ => return None,
                };
                let reverse = match *order { "asc" => false, "desc" => true, _ => return None };
                let column = if let Some(column) = parts.get(3) { Some(positive(column)?) } else { None };
                Some(Self::Sort { kind, reverse, column })
            }
            ["uniq"] => Some(Self::Unique),
            ["uniq-count"] => Some(Self::Count),
            ["head", n] => Some(Self::Head(positive(n)?)),
            ["tail", n] => Some(Self::Tail(positive(n)?)),
            ["cut", columns] => {
                let columns = columns.split(',').map(positive).collect::<Option<Vec<_>>>()?;
                Some(Self::Select(columns))
            }
            ["wc"] => Some(Self::Lines),
            [kind @ ("sum" | "mean"), key, value] => Some(Self::Reduce {
                mean: *kind == "mean", key: positive(key)?, value: positive(value)?,
            }),
            ["origin"] if action.id == "2.2" => Some(Self::Origin),
            ["decode"] if action.id == "4.6" => Some(Self::Decode),
            _ => None,
        }
    }

    fn command(&self) -> (&'static str, Vec<String>) {
        match self {
            Self::Sort { kind, reverse, column } => {
                let mut flags = String::from("-");
                if *reverse { flags.push('r'); }
                match kind {
                    SortKind::Text | SortKind::Date => {}
                    SortKind::Number => flags.push('n'),
                    SortKind::Version => flags.push('V'),
                }
                let mut args = Vec::new();
                if flags.len() > 1 { args.push(flags); }
                if let Some(column) = column {
                    args.extend(["-t".into(), "\t".into(), "-k".into(), format!("{column},{column}")]);
                }
                ("sort", args)
            }
            Self::Unique => ("uniq", vec![]),
            Self::Count => ("uniq", vec!["-c".into()]),
            Self::Head(amount) => ("head", vec!["-n".into(), amount.to_string()]),
            Self::Tail(amount) => ("tail", vec!["-n".into(), amount.to_string()]),
            Self::Select(columns) => ("cut", vec!["-f".into(),
                columns.iter().map(usize::to_string).collect::<Vec<_>>().join(",")]),
            Self::Search { mode, pattern } => {
                let options = match mode { Search::Literal => "-F", Search::IgnoreCase => "-Fi", Search::Regex => "-E", Search::Exclude => "-Fv" };
                ("grep", vec![options.into(), "--".into(), pattern.clone()])
            }
            Self::Lines => ("wc", vec!["-l".into()]),
            Self::Reduce { mean, key, value } => {
                let calculation = if *mean { "sum[k]/n[k]" } else { "sum[k]" };
                let d = '$';
                ("awk", vec!["-F".into(), "\t".into(), format!(
                    r#"BEGIN {{ OFS="	" }} {d}{key} != "" && {d}{value} ~ /^-?[0-9]+([.][0-9]+)?$/ {{ sum[{d}{key}]+={d}{value}; n[{d}{key}]++ }} END {{ for (k in sum) print k, {calculation} }}"#
                )])
            }
            Self::Origin => ("awk", vec!["-F".into(), "\t".into(),
                r#"$1 != "" { print $1; next } $2 != "" { print $2 }"#.into()]),
            Self::Decode => ("awk", vec!["-F".into(), "\t".into(), r#"
BEGIN { digits="0123456789abcdef" }
{
    hex=tolower($5); gsub(/:/, "", hex); if (hex !~ /^[0-9a-f]+$/ || length(hex)%2) next;
    label=($1!=""?$1:($2!=""?$2:"-")) " (" $3 " → " $4 "): ";
    line="";
    for(i=1;i<length(hex);i+=2) {
        b=(index(digits,substr(hex,i,1))-1)*16+index(digits,substr(hex,i+1,1))-1;
        if(b==10) { if(line!="") print label line; line=""; }
        else if(b!=13) line=line (b<32 || b==127 ? "·" : sprintf("%c",b));
    }
    if(line!="") print label line;
}"#.into()]),
        }
    }

    pub fn shell(&self) -> String {
        let (program, args) = self.command();
        let args: Vec<_> = args.iter().map(|arg| quote(arg.as_ref())).collect();
        format!("LC_ALL=C {program} {}", args.join(" ")).trim_end().to_owned()
    }
}

pub fn preview(steps: &[Step]) -> String {
    steps.iter().filter(|step| step.enabled)
        .map(|step| format!(" | {}", step.stage.shell())).collect()
}

pub fn run(input: &str, steps: &[Step]) -> io::Result<String> {
    let mut bytes = input.as_bytes().to_vec();
    for step in steps.iter().filter(|step| step.enabled) {
        let (program, args) = step.stage.command();
        let mut child = Command::new(program).args(args).env("LC_ALL", "C")
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        let mut stdin = child.stdin.take().expect("piped input");
        let writer = thread::spawn(move || stdin.write_all(&bytes));
        let output = child.wait_with_output()?;
        let write_result = writer.join().map_err(|_| io::Error::other("Pipeline input failed"))?;
        if !output.status.success() && !(program == "grep" && output.status.code() == Some(1)) {
            let reason = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::other(format!("{program}: {}{}", reason.trim(),
                output.status.signal().map_or_else(String::new, |signal| format!(" (signal {signal})")))));
        }
        if let Err(error) = write_result {
            if error.kind() != io::ErrorKind::BrokenPipe { return Err(error); }
        }
        bytes = output.stdout;
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_typed_output_steps_in_order() {
        let action = crate::actions::CATALOG.iter().find(|action| action.id == "3.7").unwrap();
        let stages: Vec<_> = ["cut 1,2", "sum 1 2", "sort number desc 2", "grep regex ^b", "tail 1"]
            .into_iter().map(|text| enabled(Stage::parse(text, action).unwrap())).collect();
        assert_eq!(run("a\t1\tx\nb\t2\ty\nb\t3\tz\n", &stages).unwrap(), "b\t5\n");
        assert_eq!(run("a\n", &[enabled(Stage::Search {
            mode: Search::Literal, pattern: "no match".into(),
        })]).unwrap(), "");
    }

    #[test]
    fn edits_are_arguments_and_never_shell_source() {
        let action = crate::actions::CATALOG.iter().find(|action| action.id == "3.7").unwrap();
        let stage = Stage::parse("grep literal echo | head", action).unwrap();
        let (program, args) = stage.command();
        assert_eq!(program, "grep");
        assert_eq!(args.last().unwrap(), "echo | head");
    }
}
