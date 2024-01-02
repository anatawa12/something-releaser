use crate::{arg_or_stdin, CmdResult};
use semver::Version;
use std::env::Args;

fn parse_version(args: &mut Args) -> CmdResult<Version> {
    Ok(Version::parse(&arg_or_stdin(args, "version")?).expect("invalid version"))
}

pub fn version_to_version_command(
    mut args: Args,
    f: impl FnOnce(&mut Args, &mut Version) -> CmdResult,
) -> CmdResult {
    let mut version = parse_version(&mut args)?;
    f(&mut args, &mut version)?;
    println!("{version}");
    ok!();
}

pub fn version_to_string_command(
    mut args: Args,
    f: impl FnOnce(&mut Args, &Version) -> CmdResult<String>,
) -> CmdResult {
    let version = parse_version(&mut args)?;
    println!("{result}", result = f(&mut args, &version)?);
    ok!();
}
