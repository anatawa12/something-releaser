use crate::{CmdResult, MaybeStdin};
use clap::Parser;
use std::convert::Infallible;
use std::env;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Parser)]
#[allow(private_interfaces)]
pub enum GithubActionsUtilities {
    GhSetOutput {
        name: String,
        #[arg(default_value_t = Default::default())]
        value: MaybeStdin<String>,
    },
    GhExportVariable {
        name: String,
        #[arg(default_value_t = Default::default())]
        value: MaybeStdin<String>,
    },
    #[command(alias = "gh-set-secret")]
    GhAddSecret {
        #[arg(default_value_t = Default::default())]
        value: MaybeStdin<String>,
    },
    GhAddPath {
        #[arg(default_value_t = Default::default())]
        path: MaybeStdin<String>,
    },
    GhGroupStart {
        #[arg(default_value_t = Default::default())]
        name: MaybeStdin<String>,
    },
    GhGroupEnd,
    GhError(GhAnnotationCommand),
    GhWarning(GhAnnotationCommand),
    GhNotice(GhAnnotationCommand),
}

#[derive(Debug, Parser)]
#[command(no_binary_name = true)]
struct GhAnnotationCommand {
    #[arg(short, long)]
    title: Option<String>,
    #[arg(short, long)]
    file: Option<String>,
    #[arg(short, long)]
    position: Option<PositionInfo>,
    value: Vec<String>,
}

#[derive(Debug, Clone)]
struct PositionInfo {
    line: Option<String>,
    end_line: Option<String>,
    col: Option<String>,
    end_column: Option<String>,
}

impl FromStr for PositionInfo {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some((first, rest)) = value.split_once(':') {
            if let Some((second, third)) = rest.split_once(':') {
                Ok(Self {
                    line: Some(first.to_string()),
                    end_line: None,
                    col: Some(second.to_string()),
                    end_column: Some(third.to_string()),
                })
            } else {
                Ok(Self {
                    line: Some(first.to_string()),
                    end_line: Some(rest.to_string()),
                    col: None,
                    end_column: None,
                })
            }
        } else {
            Ok(Self {
                line: Some(value.to_string()),
                end_line: None,
                col: None,
                end_column: None,
            })
        }
    }
}

impl GhAnnotationCommand {
    pub async fn execute(self, kind: &str) -> CmdResult {
        fn add_option(
            options: &mut Vec<(&str, String)>,
            name: &'static str,
            value: Option<String>,
        ) {
            if let Some(value) = value {
                options.push((name, value));
            }
        }

        let mut options = vec![];
        add_option(&mut options, "title", self.title);
        add_option(&mut options, "file", self.file);
        if let Some(position) = self.position {
            add_option(&mut options, "line", position.line);
            add_option(&mut options, "endLine", position.end_line);
            add_option(&mut options, "col", position.col);
            add_option(&mut options, "endColumn", position.end_column);
        }

        issue_command(kind, &options, &self.value.join(" "))
    }
}

impl GithubActionsUtilities {
    pub async fn execute(self) -> CmdResult {
        use GithubActionsUtilities::*;
        match self {
            GhSetOutput { name, value } => {
                let value = value.get("value").await?;
                if let Ok(path) = env::var("GITHUB_OUTPUT") {
                    file_command(path.as_ref(), &key_value_message(&name, &value))
                } else {
                    issue_command("set-output", &[("name", name)], &value)
                }
            }
            GhExportVariable { name, value } => {
                let value = value.get("value").await?;
                if let Ok(path) = env::var("GITHUB_ENV") {
                    file_command(path.as_ref(), &key_value_message(&name, &value))
                } else {
                    println!();
                    issue_command("set-env", &[("name", name)], &value)
                }
            }
            GhAddSecret { value } => issue_command("add-mask", &[], &value.get("secret").await?),
            GhAddPath { path } => {
                let path = path.get("path").await?;
                if let Ok(path) = env::var("GITHUB_PATH") {
                    file_command(path.as_ref(), &path)
                } else {
                    issue_command("add-path", &[], &path)
                }
            }
            GhGroupStart { name } => {
                let name = name.get("name").await?;
                issue_command("group", &[], &name)
            }
            GhGroupEnd => issue_command("endgroup", &[], ""),
            GhError(cmd) => cmd.execute("error").await,
            GhWarning(cmd) => cmd.execute("warning").await,
            GhNotice(cmd) => cmd.execute("notice").await,
        }
    }
}

fn issue_command(command: &str, options: &[(&str, String)], value: &str) -> CmdResult {
    let mut command_builder = String::from("::") + command;

    let mut options = options.iter();
    if let Some((name, value)) = options.next() {
        {
            command_builder.push(' ');
            command_builder.push_str(name);
            command_builder.push('=');
            command_builder.push_str(&escape_property(value));
        }
        for (name, value) in options {
            command_builder.push(',');
            command_builder.push_str(name);
            command_builder.push('=');
            command_builder.push_str(&escape_property(value));
        }
    }

    command_builder.push_str("::");
    command_builder.push_str(&escape_data(value));

    println!("{}", command_builder);
    ok!();

    fn escape_property(value: &str) -> String {
        escapes!(value, '%' => "%25", '\r' => "%0D", '\n' => "%0A", ':' => "%3A", ',' => "%2C")
    }

    fn escape_data(value: &str) -> String {
        escapes!(value, '%' => "%25", '\r' => "%0D", '\n' => "%0A")
    }
}

fn key_value_message(key: &str, value: &str) -> String {
    let delim = format!("delimiter={}", uuid::Uuid::new_v4());
    assert!(!value.contains(&delim));
    format!(
        "{key}<<{delim}\n{value}\n{delim}",
        key = key,
        delim = delim,
        value = value,
    )
}

fn file_command(path: &Path, value: &str) -> CmdResult {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("opening output file");
    file.write_all(value.as_bytes())
        .expect("writing output file");
    file.write_all(b"\n").expect("writing output file");
    file.flush().expect("flushing output file");
    ok!()
}
