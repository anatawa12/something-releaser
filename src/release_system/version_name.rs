use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
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
        fn parse_u8(str: &str) -> Result<(u8, &str), ParseIntError> {
            if let Some(end_digit) = str.find(|x: char| !x.is_ascii_digit()) {
                let (a, b) = str.split_at(end_digit);
                Ok((a.parse()?, b))
            } else {
                Ok((str.parse()?, ""))
            }
        }
        fn check_str<'a>(str: &'a str, c: &str) -> Option<&'a str> {
            str.strip_prefix(c)
        }
        pub(super) fn err_from_int_parse(base: ParseIntError) -> VersionNameParsingError {
            ErrorInner::IntParsing(base).into()
        }

        fn next_version_elem(s: &str) -> Result<(Option<u8>, &str), VersionNameParsingError> {
            Ok(if let Some(s) = check_str(s, ".") {
                let (elem, s) = parse_u8(s).map_err(err_from_int_parse)?;
                (Some(elem), s)
            } else {
                (None, s)
            })
        }
        let (major, s) = parse_u8(s).map_err(err_from_int_parse)?;
        let (minor, s) = next_version_elem(s)?;
        let (patch, s) = next_version_elem(s)?;
        let (snapshot, s) = if let Some(s) = check_str(s, "-SNAPSHOT") {
            (true, s)
        } else {
            (false, s)
        };

        if s != "" {
            return Err(ErrorInner::UnknownVersionSuffix.into());
        }

        if minor == Some(u8::MAX) {
            return Err(ErrorInner::VersionOutOfRange.into());
        }
        if patch == Some(u8::MAX) {
            return Err(ErrorInner::VersionOutOfRange.into());
        }

        debug_assert!(!patch.is_some() || minor.is_some());
        Ok(VersionName {
            major,
            minor: minor.unwrap_or(u8::MAX),
            patch: patch.unwrap_or(u8::MAX),
            snapshot,
        })
    }
}

#[derive(Debug)]
pub struct VersionNameParsingError {
    inner: ErrorInner,
}

#[derive(Debug)]
enum ErrorInner {
    IntParsing(ParseIntError),
    UnknownVersionSuffix,
    VersionOutOfRange,
}

impl From<ErrorInner> for VersionNameParsingError {
    fn from(inner: ErrorInner) -> Self {
        Self { inner }
    }
}

impl Error for VersionNameParsingError {}

impl Display for VersionNameParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            ErrorInner::IntParsing(inner) => inner.fmt(f),
            ErrorInner::UnknownVersionSuffix => write!(f, "unknown version suffix found"),
            ErrorInner::VersionOutOfRange => write!(f, "unsupported version name"),
        }
    }
}

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

#[cfg(test)]
mod tests {
    use crate::release_system::version_name::VersionNameParsingError;
    use crate::release_system::VersionName;

    #[test]
    fn parse_version() -> Result<(), VersionNameParsingError> {
        assert_eq!(VersionName::new_major(0, false), "0".parse()?);
        assert_eq!(VersionName::new_minor(0, 0, false), "0.0".parse()?);
        assert_eq!(VersionName::new(0, 0, 0, false), "0.0.0".parse()?);
        assert_eq!(VersionName::new_major(0, true), "0-SNAPSHOT".parse()?);
        assert_eq!(VersionName::new_minor(0, 0, true), "0.0-SNAPSHOT".parse()?);
        assert_eq!(VersionName::new(0, 0, 0, true), "0.0.0-SNAPSHOT".parse()?);
        Ok(())
    }
}
