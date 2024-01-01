use crate::CmdResult;
use semver::Version;
use std::env::Args;
use std::io::IsTerminal;

fn parse_version(args: &mut Args) -> CmdResult<Version> {
    match args.next().as_deref() {
        None if std::io::stdout().is_terminal() => {
            err!("No version specified. if you actually want to pass from stdin, pass '-' as the version");
        }
        None | Some("-") => {
            let as_str = std::io::read_to_string(std::io::stdin()).expect("reading stdin");
            let as_str = as_str.as_str().trim();
            if as_str.is_empty() {
                err!("No version specified");
            }
            Ok(Version::parse(as_str).expect("invalid version"))
        }
        Some(version) => Ok(Version::parse(version).expect("invalid version")),
    }
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
