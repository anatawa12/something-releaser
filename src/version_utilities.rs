use crate::version::{Prerelease, Version};
use crate::{CmdResult, MaybeStdin};
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[allow(private_interfaces)]
pub enum VersionUtilities {
    VersionStable(SimpleVersionCommand),
    VersionSnapshot(SimpleVersionCommand),
    VersionAlpha(ChannelCommand),
    VersionBeta(ChannelCommand),
    VersionCandidate(ChannelCommand),
    VersionMajor(SimpleVersionCommand),
    VersionMinor(SimpleVersionCommand),
    VersionPatch(SimpleVersionCommand),
    VersionGetChannel(SimpleVersionCommand),
    VersionSetChannel {
        version: MaybeStdin<Version>,
        target: SetChannelTarget,
        /// Version number in the channel, for stable and snapshot, this is ignored
        #[arg(default_value = "1")]
        num: u64,
    },
    VersionNext {
        #[arg(default_value_t = Default::default())]
        version: MaybeStdin<Version>,
        target: Option<VersionNextChannel>,
    },
}

#[derive(Debug, Parser)]
struct SimpleVersionCommand {
    #[arg(default_value_t = Default::default())]
    version: MaybeStdin<Version>,
}

#[derive(Debug, Parser)]
struct ChannelCommand {
    #[arg(default_value_t = Default::default())]
    version: MaybeStdin<Version>,
    #[arg(default_value = "1")]
    num: u64,
}

impl ChannelCommand {
    async fn run(self, channel: impl FnOnce(u64) -> Prerelease) -> CmdResult {
        let mut version = self.version.get("version").await?;
        version.pre = channel(self.num);
        println!("{}", version);

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

impl VersionUtilities {
    pub async fn execute(self) -> CmdResult {
        use VersionUtilities::*;

        match self {
            VersionStable(cmd) => {
                let mut version = cmd.version.get("version").await?;
                version.pre = Prerelease::None;
                println!("{}", version);
                ok!();
            }
            VersionSnapshot(cmd) => {
                let mut version = cmd.version.get("version").await?;
                version.pre = Prerelease::Snapshot;
                println!("{}", version);
                ok!()
            }
            VersionAlpha(v) => v.run(Prerelease::Alpha).await,
            VersionBeta(v) => v.run(Prerelease::Beta).await,
            VersionCandidate(v) => v.run(Prerelease::Candidate).await,
            VersionMajor(cmd) => {
                let mut version = cmd.version.get("version").await?;
                version.minor = None;
                version.patch = None;
                println!("{}", version);
                ok!()
            }
            VersionMinor(cmd) => {
                let mut version = cmd.version.get("version").await?;
                version.minor.get_or_insert(0);
                version.patch = None;
                println!("{}", version);
                ok!()
            }
            VersionPatch(cmd) => {
                let mut version = cmd.version.get("version").await?;
                version.minor.get_or_insert(0);
                version.patch.get_or_insert(0);
                println!("{}", version);
                ok!()
            }
            VersionGetChannel(cmd) => {
                let version = cmd.version.get("version").await?;

                let channel = match version.pre {
                    Prerelease::None => "stable",
                    Prerelease::Alpha(_) => "alpha",
                    Prerelease::Beta(_) => "beta",
                    Prerelease::Candidate(_) => "candidate",
                    Prerelease::Snapshot => "snapshot",
                };

                println!("{}", channel);

                ok!()
            }
            VersionSetChannel {
                version,
                target,
                num,
            } => {
                let mut version = version.get("version").await?;

                use SetChannelTarget::*;
                match target {
                    Alpha => version.pre = Prerelease::Alpha(num),
                    Beta => version.pre = Prerelease::Beta(num),
                    Rc => version.pre = Prerelease::Candidate(num),
                    Snapshot => version.pre = Prerelease::Snapshot,
                    Stable => version.pre = Prerelease::None,
                }

                println!("{}", version);
                ok!()
            }
            VersionNext { version, target } => {
                let mut version = version.get("version").await?;

                fn bump_pre(version: &mut Version) -> CmdResult {
                    match &mut version.pre {
                        Prerelease::None => {
                            err!("cannot bump prerelease number on stable version")
                        }
                        Prerelease::Snapshot => {
                            err!("cannot bump prerelease number on snapshot version")
                        }
                        Prerelease::Alpha(num) => *num += 1,
                        Prerelease::Beta(num) => *num += 1,
                        Prerelease::Candidate(num) => *num += 1,
                    }
                    ok!()
                }

                fn bump_optional(portion: &mut Option<u64>, name: &str) -> CmdResult {
                    let Some(portion) = portion else {
                        err!("{name} number not found while updating {name} number");
                    };
                    *portion += 1;
                    ok!()
                }

                match target {
                    None if version.pre != Prerelease::None => bump_pre(&mut version)?,
                    Some(VersionNextChannel::Prerelease) => bump_pre(&mut version)?,
                    None if version.patch.is_some() => bump_optional(&mut version.patch, "patch")?,
                    Some(VersionNextChannel::Patch) => bump_optional(&mut version.patch, "patch")?,
                    None if version.minor.is_some() => bump_optional(&mut version.minor, "minor")?,
                    Some(VersionNextChannel::Minor) => bump_optional(&mut version.minor, "minor")?,
                    None => version.major = version.major + 1,
                    Some(VersionNextChannel::Major) => version.major = version.major + 1,
                }

                println!("{}", version);
                ok!()
            }
        }
    }
}
