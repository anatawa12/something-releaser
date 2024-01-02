#[macro_use]
mod macros;
mod utils;
mod version_changer;
mod version_commands;

use crate::utils::ArgsExt;
use crate::version_changer::{parse_version_changers, VersionChangers};
use crate::version_commands::*;
use std::env::Args;
use std::iter::Peekable;
use std::num::NonZeroI32;
use std::process::exit;

#[tokio::main]
async fn main() {
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

fn version_channel(args: &mut Args, version: &mut semver::Version, channel: &str) -> CmdResult {
    let num = args.next_parsed_or(1u64);
    version.pre = semver::Prerelease::new(&format!("{channel}.{num}")).unwrap();
    version.build = semver::BuildMetadata::EMPTY;
    ok!()
}

fn version_changer(args: &mut Peekable<Args>) -> CmdResult<VersionChangers> {
    if let Some("-t" | "--target") = args.peek().map(|x| x.as_str()) {
        args.next();
        let name = args.next().expect("-t/--target requires an argument");
        let env_name = format!("RELEASE_CHANGER_{}", name.to_ascii_uppercase());
        Ok(parse_version_changers(&std::env::var(&env_name).unwrap_or_else(|_| {
            panic!("environment variable {} not set", env_name)
        })))
    } else {
        Ok(parse_version_changers(
            &std::env::var("RELEASE_CHANGER")
                .expect("environment variable RELEASE_CHANGER not set"),
        ))
    }
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
                let changers = version_changer(&mut args)?;
                let version = changers.get_version().await;
                println!("{}", version);
                ok!()
            }

            Some("set-version") => {
                let mut args = args.peekable();
                let changers = version_changer(&mut args)?;
                let version = args.next().expect("version name not found");
                changers.set_version(version).await;

                ok!()
            }

            Some(other) => err!("unknown command: {other}"),
        };
    }
}
