use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub use builder::Builder;
pub use publisher::Publisher;
pub use version_changer::VersionChanger;
pub use version_info::VersionInfo;
pub use version_name::VersionName;

use crate::*;

mod command_builder;

macro_rules! __release_system_enum {
    ($($name: ident -> $value: expr,)*) => {

#[derive(Copy, Clone)]
pub enum ReleaseSystem {
    $($name,)*
}

impl FromStr for ReleaseSystem {
    type Err = UnknownTypeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            $($value => Ok(ReleaseSystem::$name),)*
            unknown => Err(UnknownTypeErr{ name: unknown.to_owned() })
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
pub struct UnknownTypeErr {
    name: String,
}

impl Display for UnknownTypeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown type error: {}", &self.name)
    }
}

macro_rules! types_enum {
    ($trait_name: ident { $($name: ident: $str: expr),* $(,)? }) => {
        #[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash)]
        pub(super) enum Types {
            $($name,)*
        }

        impl Types {
            pub(in super::super) fn get_instance(self) -> &'static dyn $trait_name {
                match self {
                    $(Types::$name => &$name,)*
                }
            }

            #[allow(dead_code)]
            pub(in super::super) fn name(self) -> &'static str {
                match self {
                    $(Types::$name => $str,)*
                }
            }
        }

        impl ::std::str::FromStr for Types {
            type Err = super::UnknownTypeErr;

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                match s {
                    $($str => Ok(Types::$name),)*
                    unknown => Err(super::UnknownTypeErr{ name: unknown.to_owned() })
                }
            }
        }

        impl ::std::str::FromStr for &'static dyn $trait_name {
            type Err = super::UnknownTypeErr;

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                Types::from_str(s).map(|x| x.get_instance())
            }
        }
    };
}

mod builder;
mod publisher;
mod version_changer;
mod version_info;
mod version_name;

macro_rules! fns_returns_static_slice {
    (
        $type_name: ident {
            $(
            $fun_name:ident -> [$enum_type_name: ty] {
                $($enum_val_name: ident : [$($val_name: ident),* $(,)?]),*
                $(,)?
            }
            )*
        }
    ) => {
        impl $type_name {
            $(
            fn $fun_name(self) -> &'static [$enum_type_name] {
                match self {
                    $(
                    ReleaseSystem::$enum_val_name => &[
                        $(
                        <$enum_type_name>::$val_name,
                        )*
                    ],
                    )*
                }
            }
            )*
        }
    }
}

fns_returns_static_slice! {
    ReleaseSystem {
        version_changers -> [version_changer::Types] {
            Gradle: [
                GradlePropertiesVersionChanger,
            ],
            GradleIntellijPlugin: [
                GradlePropertiesVersionChanger,
            ],
            GradleMaven: [
                GradlePropertiesVersionChanger,
            ],
            GradlePlugin: [
                GradlePropertiesVersionChanger,
            ],
        }
        builders -> [builder::Types] {
            Gradle: [
                GradleBuilder,
            ],
            GradleIntellijPlugin: [
                GradleBuilder,
            ],
            GradleMaven: [
                GradleBuilder,
            ],
            GradlePlugin: [
                GradleBuilder,
            ],
        }
        publishers -> [publisher::Types] {
            Gradle: [],
            GradleIntellijPlugin: [
                GradleIntellijPublisher,
            ],
            GradleMaven: [
                GradleMavenPublisher
            ],
            GradlePlugin: [
                GradlePluginPortalPublisher,
            ],
        }
    }
}

pub fn crate_releaser_action(systems: &[ReleaseSystem]) -> ReleaserAction<'static> {
    let mut version_changers = Vec::<version_changer::Types>::new();
    let mut builders = Vec::<builder::Types>::new();
    let mut publishers = Vec::<publisher::Types>::new();

    for system in systems {
        version_changers.extend_from_slice(system.version_changers());
        builders.extend_from_slice(system.builders());
        publishers.extend_from_slice(system.publishers());
    }

    ReleaserAction {
        version_changers: version_changers
            .into_iter()
            .unique()
            .map(|x| x.get_instance())
            .collect(),
        builders: builders
            .into_iter()
            .unique()
            .map(|x| x.get_instance())
            .collect(),
        publishers: publishers
            .into_iter()
            .unique()
            .map(|x| x.get_instance())
            .collect(),
    }
}

#[derive(Clone)]
pub struct ReleaserAction<'r> {
    pub version_changers: Vec<&'r dyn VersionChanger>,
    pub builders: Vec<&'r dyn Builder>,
    pub publishers: Vec<&'r dyn Publisher>,
}

impl<'r> ReleaserAction<'r> {
    pub fn verify_exit(&self) {
        let mut errors = false;
        if self.version_changers.is_empty() {
            error!("no version changing release system specified!");
            errors = true;
        }
        if errors {
            std::process::exit(-1);
        }
    }
}
