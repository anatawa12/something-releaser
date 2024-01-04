use clap::{Parser, ValueEnum};
use semver::Version;
use crate::{CmdResult, MaybeStdin};

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

impl VersionUtilities {
    pub async fn execute(self) -> CmdResult {
        use VersionUtilities::*;

        match self {
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
        }
    }
}
