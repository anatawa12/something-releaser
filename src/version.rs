use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub pre: Prerelease,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;
        if let Some(minor) = self.minor {
            write!(f, ".{}", minor)?;
            if let Some(patch) = self.patch {
                write!(f, ".{}", patch)?;
            }
        }
        match self.pre {
            Prerelease::None => {}
            Prerelease::Alpha(num) => write!(f, "-alpha.{}", num)?,
            Prerelease::Beta(num) => write!(f, "-beta.{}", num)?,
            Prerelease::Candidate(num) => write!(f, "-rc.{}", num)?,
            Prerelease::Snapshot => write!(f, "-SNAPSHOT")?,
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Prerelease {
    None,
    Alpha(u64),
    Beta(u64),
    Candidate(u64),
    Snapshot,
}

#[derive(Debug)]
pub struct InvalidVersion(());

impl From<std::num::ParseIntError> for InvalidVersion {
    fn from(_: std::num::ParseIntError) -> Self {
        Self(())
    }
}

impl std::error::Error for InvalidVersion {}

impl Display for InvalidVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid version")
    }
}

impl FromStr for Version {
    type Err = InvalidVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (stable, prerelease) = s
            .split_once('-')
            .map(|(a, b)| (a, Some(b)))
            .unwrap_or((s, None));
        if let Some((major, rest)) = stable.split_once('.') {
            if let Some((minor, patch)) = rest.split_once('.') {
                Ok(Self {
                    major: major.parse()?,
                    minor: Some(minor.parse()?),
                    patch: Some(patch.parse()?),
                    pre: parse_prerelease(prerelease)?,
                })
            } else {
                Ok(Self {
                    major: major.parse()?,
                    minor: Some(rest.parse()?),
                    patch: None,
                    pre: parse_prerelease(prerelease)?,
                })
            }
        } else {
            Ok(Self {
                major: stable.parse()?,
                minor: None,
                patch: None,
                pre: parse_prerelease(prerelease)?,
            })
        }
    }
}

fn parse_prerelease(s: Option<&str>) -> Result<Prerelease, InvalidVersion> {
    let Some(s) = s else {
        return Ok(Prerelease::None);
    };

    if s == "SNAPSHOT" {
        return Ok(Prerelease::Snapshot);
    }

    let Some((channel, num)) = s.split_once('.') else {
        return Err(InvalidVersion(()));
    };

    match channel {
        "alpha" => Ok(Prerelease::Alpha(num.parse()?)),
        "beta" => Ok(Prerelease::Beta(num.parse()?)),
        "rc" => Ok(Prerelease::Candidate(num.parse()?)),
        _ => Err(InvalidVersion(())),
    }
}
