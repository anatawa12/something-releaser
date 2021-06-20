use std::str::FromStr;
use std::fmt::{Display, Formatter};

pub enum ReleaseSystem {
}

#[derive(Debug)]
pub struct ReleaseSystemErr {
    name: String,
}

impl FromStr for ReleaseSystem {
    type Err = ReleaseSystemErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            unknown => Err(ReleaseSystemErr{ name: unknown.to_owned() })
        }
    }
}

impl Display for ReleaseSystemErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown release system error: {}", &self.name)
    }
}
