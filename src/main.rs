#[macro_use]
mod macros;
mod commands;
mod utils;
mod version_changer;
mod version_commands;

use crate::commands::gradle_maven::GradleMaven;
use crate::commands::gradle_plugin_portal::GradlePluginPortal;
use crate::commands::gradle_signing::GradleSigning;
use crate::utils::ArgsExt;
use crate::version_changer::{parse_version_changers, VersionChangers};
use crate::version_commands::*;
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::env::Args;
use std::io::{IsTerminal, Write};
use std::iter::Peekable;
use std::num::NonZeroI32;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use crate::commands::gradle_intellij::GradleIntellij;
use crate::commands::publish_to_curse_forge::PublishToCurseForge;
use crate::commands::send_discord::SendDiscord;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    exit(match do_main(std::env::args()).await {
        Ok(()) => 0,
        Err(e) => e.get(),
    })
}

type CmdResult<T = ()> = Result<T, NonZeroI32>;

fn sanitize_cmd(mut cmd: &str) -> &str {
    if cfg!(windows) {
        cmd = cmd.strip_suffix(".exe").unwrap_or(cmd)
    }
    if let Some(slash) = cmd.rfind('/') {
        cmd = &cmd[slash + 1..]
    }
    cmd
}

fn arg_or_stdin(args: &mut impl Iterator<Item = String>, kind: &str) -> CmdResult<String> {
    fn inner(value: Option<String>, kind: &str) -> CmdResult<String> {
        if let Some(arg) = value {
            if arg != "-" {
                return Ok(arg);
            }
        } else if std::io::stdin().is_terminal() {
            err!(
                "No {} specified. if you actually want to pass from stdin, pass '-' as the version",
                kind
            );
        }

        read_stdin()
    }

    inner(args.next(), kind)
}

fn read_stdin() -> CmdResult<String> {
    let mut read = std::io::read_to_string(std::io::stdin()).expect("reading stdin");
    if read.ends_with('\n') {
        read.pop();
    }
    if read.ends_with('\r') {
        read.pop();
    }
    Ok(read)
}

fn version_channel(args: &mut Args, version: &mut semver::Version, channel: &str) -> CmdResult {
    let num = args.next_parsed_or(1u64);
    version.pre = semver::Prerelease::new(&format!("{channel}.{num}")).unwrap();
    version.build = semver::BuildMetadata::EMPTY;
    ok!()
}

#[derive(Debug, Default, Deserialize)]
struct ConfigFile {
    #[serde(rename = "releaseChanger", default)]
    release_changer: Option<VersionChangers>,
    #[serde(rename = "target", default)]
    targets: HashMap<String, TargetConfig>,
}

#[derive(Debug, Deserialize)]
struct TargetConfig {
    #[serde(rename = "releaseChanger", default)]
    release_changer: Option<VersionChangers>,
}

async fn env_file() -> ConfigFile {
    if let Some(json) = read_or_none(".something-releaser.json").await {
        return serde_json::from_str(&json).expect("parsing .something-releaser.json");
    }

    return Default::default();
    // tokio::fs::read_to_string(".something-releaser.json")
    async fn read_or_none(path: &str) -> Option<String> {
        match tokio::fs::read_to_string(path).await {
            Ok(s) => Some(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => panic!("reading {}: {}", path, e),
        }
    }
}

async fn version_changer(args: &mut Peekable<Args>) -> CmdResult<VersionChangers> {
    let mut env = env_file().await;

    if let Some("-t" | "--target") = args.peek().map(|x| x.as_str()) {
        args.next();
        let name = args.next().expect("-t/--target requires an argument");
        Ok(env
            .targets
            .get_mut(&name)
            .and_then(|x| x.release_changer.as_mut())
            .map(std::mem::take)
            .unwrap_or_else(|| {
                let env_name = format!("RELEASE_CHANGER_{}", name.to_ascii_uppercase());
                parse_version_changers(
                    &env::var(&env_name)
                        .unwrap_or_else(|_| panic!("environment variable {} not set", env_name)),
                )
            }))
    } else {
        Ok(env.release_changer.unwrap_or_else(|| {
            parse_version_changers(
                &env::var("RELEASE_CHANGER").expect("environment variable RELEASE_CHANGER not set"),
            )
        }))
    }
}

fn gh_issue_command(command: &str, options: &[(&str, String)], value: &str) -> CmdResult {
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

fn gh_key_value_message(key: &str, value: &str) -> String {
    let delim = format!("delimiter={}", uuid::Uuid::new_v4());
    assert!(!value.contains(&delim));
    format!(
        "{key}<<{delim}\n{value}\n{delim}",
        key = key,
        delim = delim,
        value = value,
    )
}

fn gh_file_command(path: &Path, value: &str) -> CmdResult {
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

fn gh_annotation_command(kind: &str, args: &mut Args) -> CmdResult {
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

    let args = GhAnnotationCommand::parse_from(args);

    fn add_option(options: &mut Vec<(&str, String)>, name: &'static str, value: Option<String>) {
        if let Some(value) = value {
            options.push((name, value));
        }
    }

    let mut options = vec![];
    add_option(&mut options, "title", args.title);
    add_option(&mut options, "file", args.file);
    if let Some(position) = args.position {
        add_option(&mut options, "line", position.line);
        add_option(&mut options, "endLine", position.end_line);
        add_option(&mut options, "col", position.col);
        add_option(&mut options, "endColumn", position.end_column);
    }

    gh_issue_command(kind, &options, &args.value.join(" "))
}

async fn do_main(mut args: Args) -> CmdResult<()> {
    loop {
        return match args.next().as_deref().map(sanitize_cmd) {
            None => err!("No command specified"),
            Some("something-releaser") => continue,
            Some("version-stable") => version_to_version_command(args, |_, version| {
                version.pre = semver::Prerelease::EMPTY;
                version.build = semver::BuildMetadata::EMPTY;
                ok!()
            }),
            Some("version-snapshot") => version_to_version_command(args, |_, version| {
                version.pre = semver::Prerelease::new("SNAPSHOT").unwrap();
                version.build = semver::BuildMetadata::EMPTY;
                ok!()
            }),
            Some("version-alpha") => version_to_version_command(args, |args, version| {
                version_channel(args, version, "alpha")
            }),
            Some("version-beta") => version_to_version_command(args, |args, version| {
                version_channel(args, version, "beta")
            }),
            Some("version-candidate") => version_to_version_command(args, |args, version| {
                version_channel(args, version, "rc")
            }),

            Some("version-major") => {
                version_to_string_command(args, |_, version| Ok(format!("{}", version.major)))
            }
            Some("version-minor") => version_to_string_command(args, |_, version| {
                Ok(format!("{}.{}", version.major, version.minor))
            }),
            Some("version-get-channel") => version_to_string_command(args, |_, version| {
                if version.pre.is_empty() {
                    return Ok("stable".to_string());
                }

                let Some((ty, rest)) = version.pre.split_once('.') else {
                    err!("invalid prerelease name: {}", version.pre);
                };
                if !rest.as_bytes().iter().all(|x| x.is_ascii_digit()) {
                    err!("invalid prerelease name: {}", version.pre);
                }
                if !matches!(ty, "alpha" | "beta" | "rc") {
                    err!("invalid prerelease name: {}", version.pre);
                }

                Ok(ty.to_string())
            }),

            Some("version-set-channel") => {
                version_to_version_command(args, |args, version| match args.next().as_deref() {
                    None => err!("no channel specified"),
                    Some("a" | "alpha" | "α") => version_channel(args, version, "alpha"),
                    Some("b" | "beta" | "β") => version_channel(args, version, "beta"),
                    Some("rc" | "candidate") => version_channel(args, version, "rc"),
                    Some("snapshot") => {
                        version.pre = semver::Prerelease::new("SNAPSHOT").unwrap();
                        version.build = semver::BuildMetadata::EMPTY;
                        ok!()
                    }
                    Some("stable") => {
                        version.pre = semver::Prerelease::EMPTY;
                        version.build = semver::BuildMetadata::EMPTY;
                        ok!()
                    }
                    Some(other) => err!("unknown release channel: {other}"),
                })
            }

            Some("version-next") => version_to_version_command(args, |args, version| {
                fn bump_pre(version: &mut semver::Version) -> CmdResult {
                    let Some((before, rest)) = version.pre.rsplit_once('.') else {
                        err!("invalid prerelease name: {}", version.pre);
                    };
                    let number = rest.parse::<u64>().expect("invalid prerelease number");
                    let number = number.checked_add(1).expect("prerelease number overflow");
                    version.pre =
                        semver::Prerelease::new(&format!("{}.{}", before, number)).unwrap();
                    ok!()
                }
                match args.next().as_deref() {
                    None if !version.pre.is_empty() => bump_pre(version)?,
                    None => {
                        version.patch = version.patch.checked_add(1).expect("patch number overflow")
                    }
                    Some(
                        "pre" | "prerelease" | "a" | "alpha" | "α" | "b" | "beta" | "β" | "rc"
                        | "candidate" | "snapshot",
                    ) => bump_pre(version)?,
                    Some("pat" | "patch") => {
                        version.patch = version.patch.checked_add(1).expect("patch number overflow")
                    }
                    Some("min" | "minor") => {
                        version.minor = version.minor.checked_add(1).expect("minor number overflow")
                    }
                    Some("maj" | "major") => {
                        version.major = version.major.checked_add(1).expect("major number overflow")
                    }
                    Some(other) => err!("unknown next version target: {other}"),
                }
                ok!()
            }),

            Some("get-version") => {
                let mut args = args.peekable();
                let changers = version_changer(&mut args).await?;
                let version = changers.get_version().await;
                println!("{}", version);
                ok!()
            }

            Some("set-version") => {
                let mut args = args.peekable();
                let changers = version_changer(&mut args).await?;
                let version = arg_or_stdin(&mut args, "version")?;
                changers.set_version(version).await;

                ok!()
            }

            // configure utilities
            Some("prepare-gradle-maven") => {
                let mut url = None;
                let mut user = None;
                let mut pass = None;
                while let Some(arg) = args.next() {
                    match arg.as_str() {
                        "--url" => {
                            url = Some(args.next().expect("value for --url not found"));
                        }
                        "--user" => {
                            user = Some(args.next().expect("value for --user not found"));
                        }
                        "--pass" => {
                            pass = Some(args.next().expect("value for --pass not found"));
                        }
                        "--" => break,
                        opt if opt.starts_with('-') => err!("unknown option: {opt}"),
                        _ => url = Some(arg),
                    }
                }

                GradleMaven {
                    url: url.unwrap_or_else(|| args.next().expect("url not specified")),
                    user: user.unwrap_or_default(),
                    pass: pass.unwrap_or_default(),
                }
                .configure()
                .await;

                ok!()
            }
            Some("prepare-gradle-signing") => {
                let key = args.next().expect("no key specified");
                let pass = args
                    .next()
                    .expect("no GPG pass specified. if not exists, pass empty");
                GradleSigning { key, pass }.configure().await;
                ok!()
            }
            Some("prepare-gradle-plugin-portal") => {
                GradlePluginPortal::parse_from(args).configure().await;
                ok!()
            }
            Some("prepare-gradle-intellij") => {
                GradleIntellij::parse_from(args).configure().await;
                ok!()
            }

            // api utilities
            Some("publish-to-curse-forge") => {
                PublishToCurseForge::parse_from(args).run().await
            }
            Some("send-discord") => {
                SendDiscord::parse_from(args).run().await
            }

            // github utils
            Some("gh-set-output") => {
                let name = args.next().expect("output name not found");
                let value = arg_or_stdin(&mut args, "value")?;
                if let Ok(path) = env::var("GITHUB_OUTPUT") {
                    gh_file_command(path.as_ref(), &gh_key_value_message(&name, &value))
                } else {
                    gh_issue_command("set-output", &[("name", name)], &value)
                }
            }
            Some("gh-export-variable") => {
                let name = args.next().expect("output name not found");
                let value = arg_or_stdin(&mut args, "value")?;
                if let Ok(path) = env::var("GITHUB_ENV") {
                    gh_file_command(path.as_ref(), &gh_key_value_message(&name, &value))
                } else {
                    println!();
                    gh_issue_command("set-env", &[("name", name)], &value)
                }
            }
            Some("gh-set-secret") => {
                gh_issue_command("add-mask", &[], &arg_or_stdin(&mut args, "secret")?)
            }
            Some("gh-add-path") => {
                let value = arg_or_stdin(&mut args, "path")?;
                if let Ok(path) = env::var("GITHUB_PATH") {
                    gh_file_command(path.as_ref(), &value)
                } else {
                    gh_issue_command("add-path", &[], &value)
                }
            }
            Some("gh-group-start") => {
                let value = arg_or_stdin(&mut args, "group name")?;
                gh_issue_command("group", &[], &value)
            }
            Some("gh-group-end") => gh_issue_command("group-end", &[], ""),
            Some("gh-error") => gh_annotation_command("error", &mut args),
            Some("gh-warning") => gh_annotation_command("notice", &mut args),
            Some("gh-notice") => gh_annotation_command("info", &mut args),

            Some(other) => err!("unknown command: {other}"),
        };
    }
}
