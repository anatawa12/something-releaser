use std::fmt::{Display, Formatter};
use std::str::FromStr;

// if minor, patch are u16::MAX, it means no value exists
// major must exist.
// if patch exists, minor must exist.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct VersionName {
    major: u8,
    minor: u8,
    patch: u8,
    snapshot: bool,
}

#[allow(dead_code)]
impl VersionName {
    pub fn new(major: u8, minor: u8, patch: u8, snapshot: bool) -> Self {
        assert_ne!(minor, u8::MAX, "minor must not u8::MAX");
        assert_ne!(patch, u8::MAX, "minor must not u8::MAX");
        Self {
            major,
            minor,
            patch,
            snapshot,
        }
    }

    pub fn new_minor(major: u8, minor: u8, snapshot: bool) -> Self {
        assert_ne!(minor, u8::MAX, "minor must not u8::MAX");
        Self {
            major,
            minor,
            patch: u8::MAX,
            snapshot,
        }
    }

    pub fn new_major(major: u8, snapshot: bool) -> Self {
        Self {
            major,
            minor: u8::MAX,
            patch: u8::MAX,
            snapshot,
        }
    }

    pub fn make_next_patch(self) -> Self {
        Self {
            patch: self.patch + 1,
            ..self
        }
    }

    pub fn un_snapshot(self) -> Self {
        Self {
            snapshot: false,
            ..self
        }
    }

    pub fn of_snapshot(self) -> Self {
        Self {
            snapshot: true,
            ..self
        }
    }

    pub fn major(self) -> u8 {
        self.major
    }

    pub fn minor(self) -> Option<u8> {
        if self.minor == u8::MAX {
            None
        } else {
            Some(self.minor)
        }
    }

    pub fn patch(self) -> Option<u8> {
        if self.patch == u8::MAX {
            None
        } else {
            Some(self.patch)
        }
    }

    pub fn snapshot(self) -> bool {
        self.snapshot
    }
}

impl FromStr for VersionName {
    type Err = VersionNameParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn split_dot<'s>(str: &'s str) -> Result<(Option<u8>, &'s str), std::num::ParseIntError> {
            if let Some((value, str)) = str.split_once(".") {
                Ok((Some(value.parse()?), str))
            } else {
                Ok((None, str))
            }
        }
        fn parse(s: &str) -> Option<VersionName> {
            let (major, s) = split_dot(s).ok()?;
            let major = major?;
            let (minor, s) = split_dot(s).ok()?;
            let (patch, s) = split_dot(s).ok()?;
            let snapshot = match s {
                "" => false,
                "-SNAPSHOT" => true,
                _ => return None,
            };

            if minor == Some(u8::MAX) {
                return None;
            }
            if patch == Some(u8::MAX) {
                return None;
            }

            debug_assert!(!patch.is_some() || minor.is_some());
            Some(VersionName {
                major,
                minor: minor.unwrap_or(u8::MAX),
                patch: patch.unwrap_or(u8::MAX),
                snapshot,
            })
        }
        match parse(s) {
            Some(v) => Ok(v),
            None => Err(VersionNameParsingError()),
        }
    }
}

pub struct VersionNameParsingError();

impl Display for VersionName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.patch != u8::MAX {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        } else if self.minor != u8::MAX {
            write!(f, "{}.{}", self.major, self.minor)?;
        } else {
            write!(f, "{}", self.major)?;
        }
        if self.snapshot {
            write!(f, "-SNAPSHOT")?;
        }
        Ok(())
    }
}
