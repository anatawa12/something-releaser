#[macro_use]
mod macros;
mod commands;
mod utils;
mod version_changer;

use crate::commands::gradle_intellij::GradleIntellij;
use crate::commands::gradle_maven::GradleMaven;
use crate::commands::gradle_plugin_portal::GradlePluginPortal;
use crate::commands::gradle_signing::GradleSigning;
use crate::commands::publish_to_curse_forge::PublishToCurseForge;
use crate::commands::send_discord::SendDiscord;
use crate::version_changer::{parse_version_changers, VersionChangers};
use clap::{Parser, ValueEnum};
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::io::{IsTerminal, Write};
use std::num::NonZeroI32;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    exit(match Frontend::parse().execute().await {
        Ok(()) => 0,
        Err(e) => e.get(),
    })
}

#[derive(Debug, Parser)]
#[command(multicall = true)]
enum Frontend {
    #[command(flatten)]
    Direct(Commands),
    #[clap(subcommand)]
    #[command(name = "something-releaser")]
    AsArgument(Commands),
}

impl Frontend {
    pub(crate) async fn execute(self) -> CmdResult {
        match self {
            Frontend::Direct(commands) => commands.execute().await,
            Frontend::AsArgument(commands) => commands.execute().await,
        }
    }
}

type CmdResult<T = ()> = Result<T, NonZeroI32>;

#[derive(Debug, Clone)]
enum MaybeStdin<T: FromStr> {
    Stdin,
    Value(T),
}

impl<T: FromStr> FromStr for MaybeStdin<T> {
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(Self::Stdin)
        } else {
            Ok(Self::Value(s.parse()?))
        }
    }
}

impl<T> MaybeStdin<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    pub async fn get(self, kind: &str) -> CmdResult<T> {
        match self {
            Self::Stdin if std::io::stdin().is_terminal() => {
                err!("No {} specified. if you actually want to pass from stdin, pass '-' as the version", kind);
            }
            Self::Stdin => match read_stdin().await?.parse() {
                Ok(value) => Ok(value),
                Err(e) => {
                    err!("invalid {}: {}", kind, e);
                }
            },
            Self::Value(value) => Ok(value),
        }
    }
}

async fn read_stdin() -> CmdResult<String> {
    let mut read = String::new();
    tokio::io::stdin()
        .read_to_string(&mut read)
        .await
        .expect("reading stdin");
    if read.ends_with('\n') {
        read.pop();
    }
    if read.ends_with('\r') {
        read.pop();
    }
    Ok(read)
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

#[derive(Debug, Parser)]
enum Commands {
    // version utilities
    VersionStable(SimpleVersionCommand),
    VersionSnapshot(SimpleVersionCommand),
    VersionAlpha(ChannelCommand),
    VersionBeta(ChannelCommand),
    VersionCandidate(ChannelCommand),
    VersionMajor(SimpleVersionCommand),
    VersionMinor(SimpleVersionCommand),
    VersionGetChannel(SimpleVersionCommand),
    VersionSetChannel {
        version: MaybeStdin<Version>,
        target: SetChannelTarget,
        /// Version number in the channel, for stable and snapshot, this is ignored
        #[arg(default_value = "1")]
        num: u64,
    },
    VersionNext {
        #[command(flatten)]
        version: SimpleVersionCommand,
        target: Option<VersionNextChannel>,
    },

    // changer commands
    GetVersion(ChangerCommand),
    SetVersion {
        #[command(flatten)]
        changer: ChangerCommand,
        #[clap(default_value = "-")]
        version: MaybeStdin<String>,
    },

    // configure utilities
    PrepareGradleMaven(GradleMaven),
    PrepareGradleSigning(GradleSigning),
    PrepareGradlePluginPortal(GradlePluginPortal),
    PrepareGradleIntellij(GradleIntellij),

    // api utilities
    PublishToCurseForge(PublishToCurseForge),
    SendDiscord(SendDiscord),

    // github actions utils
    GhSetOutput {
        name: String,
        #[clap(default_value = "-")]
        value: MaybeStdin<String>,
    },
    GhExportVariable {
        name: String,
        #[clap(default_value = "-")]
        value: MaybeStdin<String>,
    },
    #[command(alias = "gh-set-secret")]
    GhAddSecret {
        #[clap(default_value = "-")]
        value: MaybeStdin<String>,
    },
    GhAddPath {
        #[clap(default_value = "-")]
        path: MaybeStdin<String>,
    },
    GhGroupStart {
        #[clap(default_value = "-")]
        name: MaybeStdin<String>,
    },
    GhGroupEnd,
    GhError(GhAnnotationCommand),
    GhWarning(GhAnnotationCommand),
    GhNotice(GhAnnotationCommand),
}

#[derive(Debug, Parser)]
struct SimpleVersionCommand {
    #[clap(default_value = "-")]
    version: MaybeStdin<Version>,
}

impl SimpleVersionCommand {
    async fn v2v(self, f: impl FnOnce(&mut Version) -> CmdResult) -> CmdResult {
        let mut version = self.version.get("version").await?;
        f(&mut version)?;
        println!("{}", version);
        ok!()
    }

    async fn v2str(self, f: impl FnOnce(&mut Version) -> CmdResult<String>) -> CmdResult {
        let mut version = self.version.get("version").await?;
        f(&mut version)?;
        println!("{}", version);
        ok!()
    }
}

#[derive(Debug, Parser)]
struct ChannelCommand {
    #[command(flatten)]
    version: SimpleVersionCommand,
    #[arg(default_value = "1")]
    num: u64,
}

impl ChannelCommand {
    async fn run(self, channel: &str) -> CmdResult {
        self.version
            .v2v(|version| Self::exec(version, channel, self.num))
            .await
    }

    fn exec(version: &mut Version, channel: &str, num: u64) -> CmdResult {
        version.pre = semver::Prerelease::new(&format!("{channel}.{num}")).unwrap();
        version.build = semver::BuildMetadata::EMPTY;
        ok!()
    }
}

#[derive(Debug, Parser)]
struct VersionSetChannel {
    #[command(flatten)]
    version: SimpleVersionCommand,
    target: SetChannelTarget,
    /// Version number in the channel, for stable and snapshot, this is ignored
    #[arg(default_value = "1")]
    num: u64,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum SetChannelTarget {
    #[value(name = "alpha", alias = "a", alias = "α")]
    Alpha,
    #[value(name = "beta", alias = "b", alias = "β")]
    Beta,
    #[value(name = "rc", alias = "candidate")]
    Rc,
    #[value(name = "snapshot")]
    Snapshot,
    #[value(name = "stable")]
    Stable,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum VersionNextChannel {
    #[value(name = "prerelease")]
    #[value(aliases = ["pre", "a", "alpha", "α", "b", "beta", "β", "rc", "candidate", "snapshot"])]
    Prerelease,
    #[value(name = "patch", alias = "pat")]
    Patch,
    #[value(name = "minor", alias = "min")]
    Minor,
    #[value(name = "major", alias = "maj")]
    Major,
}

#[derive(Debug, Parser)]
struct ChangerCommand {
    #[arg(short, long)]
    target: Option<String>,
}

impl ChangerCommand {
    pub async fn get_changer(&self) -> CmdResult<VersionChangers> {
        let mut env = env_file().await;

        if let Some(name) = &self.target {
            Ok(env
                .targets
                .get_mut(name)
                .and_then(|x| x.release_changer.as_mut())
                .map(std::mem::take)
                .unwrap_or_else(|| {
                    let env_name = format!("RELEASE_CHANGER_{}", name.to_ascii_uppercase());
                    parse_version_changers(
                        &env::var(&env_name).unwrap_or_else(|_| {
                            panic!("environment variable {} not set", env_name)
                        }),
                    )
                }))
        } else {
            Ok(env.release_changer.unwrap_or_else(|| {
                parse_version_changers(
                    &env::var("RELEASE_CHANGER")
                        .expect("environment variable RELEASE_CHANGER not set"),
                )
            }))
        }
    }
}

fn set_channel_stable(version: &mut Version) -> CmdResult {
    version.pre = semver::Prerelease::EMPTY;
    version.build = semver::BuildMetadata::EMPTY;
    ok!()
}

fn set_channel_snapshot(version: &mut Version) -> CmdResult {
    version.pre = semver::Prerelease::new("SNAPSHOT").unwrap();
    version.build = semver::BuildMetadata::EMPTY;
    ok!()
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

        gh_issue_command(kind, &options, &self.value.join(" "))
    }
}

impl Commands {
    async fn execute(self) -> CmdResult {
        use Commands::*;
        match self {
            // version utilities
            VersionStable(v) => v.v2v(set_channel_stable).await,
            VersionSnapshot(v) => v.v2v(set_channel_snapshot).await,
            VersionAlpha(v) => v.run("alpha").await,
            VersionBeta(v) => v.run("beta").await,
            VersionCandidate(v) => v.run("rc").await,
            VersionMajor(v) => v.v2str(|version| Ok(format!("{}", version.major))).await,
            VersionMinor(v) => {
                v.v2str(|version| Ok(format!("{}.{}", version.major, version.minor)))
                    .await
            }
            VersionGetChannel(v) => {
                v.v2str(|version| {
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
                })
                .await
            }
            VersionSetChannel {
                version,
                target,
                num,
            } => {
                let mut version = version.get("version").await?;

                use SetChannelTarget::*;
                match target {
                    Alpha => ChannelCommand::exec(&mut version, "alpha", num)?,
                    Beta => ChannelCommand::exec(&mut version, "beta", num)?,
                    Rc => ChannelCommand::exec(&mut version, "rc", num)?,
                    Snapshot => set_channel_snapshot(&mut version)?,
                    Stable => set_channel_stable(&mut version)?,
                }

                println!("{}", version);
                ok!()
            }
            VersionNext { version, target } => {
                version
                    .v2v(|version| {
                        use VersionNextChannel::*;
                        fn bump_pre(version: &mut Version) -> CmdResult {
                            let Some((before, rest)) = version.pre.rsplit_once('.') else {
                                err!("invalid prerelease name: {}", version.pre);
                            };
                            let number = rest.parse::<u64>().expect("invalid prerelease number");
                            let number = number.checked_add(1).expect("prerelease number overflow");
                            version.pre =
                                semver::Prerelease::new(&format!("{}.{}", before, number)).unwrap();
                            ok!()
                        }
                        match target {
                            None if !version.pre.is_empty() => bump_pre(version)?,
                            None => {
                                version.patch =
                                    version.patch.checked_add(1).expect("patch number overflow")
                            }
                            Some(Prerelease) => bump_pre(version)?,
                            Some(Patch) => {
                                version.patch =
                                    version.patch.checked_add(1).expect("patch number overflow")
                            }
                            Some(Minor) => {
                                version.minor =
                                    version.minor.checked_add(1).expect("minor number overflow")
                            }
                            Some(Major) => {
                                version.major =
                                    version.major.checked_add(1).expect("major number overflow")
                            }
                        }
                        ok!()
                    })
                    .await
            }

            // changer commands
            GetVersion(changer) => {
                println!("{}", changer.get_changer().await?.get_version().await);
                ok!()
            }
            SetVersion { changer, version } => {
                changer
                    .get_changer()
                    .await?
                    .set_version(version.get("version").await?)
                    .await;
                ok!()
            }

            // configure utilities
            PrepareGradleMaven(cmd) => cmd.configure().await,
            PrepareGradleSigning(cmd) => cmd.configure().await,
            PrepareGradlePluginPortal(cmd) => cmd.configure().await,
            PrepareGradleIntellij(cmd) => cmd.configure().await,

            // api utilities
            PublishToCurseForge(cmd) => cmd.run().await,
            SendDiscord(cmd) => cmd.run().await,

            // github actions utils
            GhSetOutput { name, value } => {
                let value = value.get("value").await?;
                if let Ok(path) = env::var("GITHUB_OUTPUT") {
                    gh_file_command(path.as_ref(), &gh_key_value_message(&name, &value))
                } else {
                    gh_issue_command("set-output", &[("name", name)], &value)
                }
            }
            GhExportVariable { name, value } => {
                let value = value.get("value").await?;
                if let Ok(path) = env::var("GITHUB_ENV") {
                    gh_file_command(path.as_ref(), &gh_key_value_message(&name, &value))
                } else {
                    println!();
                    gh_issue_command("set-env", &[("name", name)], &value)
                }
            }
            GhAddSecret { value } => gh_issue_command("add-mask", &[], &value.get("secret").await?),
            GhAddPath { path } => {
                let path = path.get("path").await?;
                if let Ok(path) = env::var("GITHUB_PATH") {
                    gh_file_command(path.as_ref(), &path)
                } else {
                    gh_issue_command("add-path", &[], &path)
                }
            }
            GhGroupStart { name } => {
                let name = name.get("name").await?;
                gh_issue_command("group", &[], &name)
            }
            GhGroupEnd => gh_issue_command("endgroup", &[], ""),
            GhError(cmd) => cmd.execute("error").await,
            GhWarning(cmd) => cmd.execute("warning").await,
            GhNotice(cmd) => cmd.execute("notice").await,
        }
    }
}
