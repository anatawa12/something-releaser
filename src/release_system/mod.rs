use std::fmt::{Display, Formatter};
use std::str::FromStr;

macro_rules! __release_system_enum {
    ($($name: ident -> $value: expr,)*) => {

pub enum ReleaseSystem {
    $($name,)*
}

impl FromStr for ReleaseSystem {
    type Err = ReleaseSystemErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            $($value => Ok(ReleaseSystem::$name),)*
            unknown => Err(ReleaseSystemErr{ name: unknown.to_owned() })
        }
    }
}
    };
}

__release_system_enum! {
    Gradle -> "gradle",
    GradleIntellijPlugin -> "gradle-intellij-plugin",
    GradleMaven -> "gradle-maven",
    GradlePlugin -> "gradle-plugin",
}

#[derive(Debug)]
pub struct ReleaseSystemErr {
    name: String,
}

impl Display for ReleaseSystemErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown release system error: {}", &self.name)
    }
}
