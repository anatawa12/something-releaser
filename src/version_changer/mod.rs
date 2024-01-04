mod command;
mod gradle_properties;
mod npm_package_json;
mod regex_pattern;

use serde::de::value::SeqAccessDeserializer;
use serde::de::SeqAccess;
use serde::{Deserialize, Deserializer};
use std::fmt::{Debug, Display};
use std::future::Future;
use std::pin::Pin;

pub(crate) use command::VersionChangerCommand;

pub(crate) trait VersionChanger: Display + Debug {
    fn parse(info: Option<&str>, path: Option<&str>) -> Self;
    async fn load_version(&self) -> String;
    async fn set_version(&self, version: &str);
}

pub(crate) trait DynVersionChanger: Display + Debug {
    fn load_version(&self) -> Pin<Box<dyn Future<Output = String> + '_>>;
    fn set_version<'a>(&'a self, version: &'a str) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
}

impl<T: VersionChanger> DynVersionChanger for T {
    fn load_version(&self) -> Pin<Box<dyn Future<Output = String> + '_>> {
        Box::pin(self.load_version())
    }

    fn set_version<'a>(&'a self, version: &'a str) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(self.set_version(version))
    }
}

#[derive(Default, Debug)]
pub(crate) struct VersionChangers {
    changers: Vec<Box<dyn DynVersionChanger>>,
}

impl VersionChangers {
    pub async fn get_version(&self) -> String {
        let mut version = None;
        for changer in &self.changers {
            let new_version = changer.load_version().await;
            if let Some(old_version) = &version {
                if old_version != &new_version {
                    panic!("version mismatch: {} != {}", old_version, new_version);
                }
            } else {
                version = Some(new_version);
            }
        }
        version.expect("no version changers")
    }

    pub async fn set_version(&self, version: String) {
        for changer in &self.changers {
            changer.set_version(&version).await;
        }
    }
}

impl<'de> Deserialize<'de> for VersionChangers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionChangersVisitor;

        impl<'de> serde::de::Visitor<'de> for VersionChangersVisitor {
            type Value = VersionChangers;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a version changers")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(parse_version_changers(v))
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Ok(VersionChangers {
                    changers: Deserialize::deserialize(SeqAccessDeserializer::new(seq))?,
                })
            }
        }

        deserializer.deserialize_any(VersionChangersVisitor)
    }
}

impl<'de> Deserialize<'de> for Box<dyn DynVersionChanger> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use KnownChanger::*;
        use Reprs::*;
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Reprs {
            InString(String),
            Tuple1((String,)),
            Tuple2((String, String)),
            Tuple3((String, String, String)),
            AsStruct(KnownChanger),
        }

        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum KnownChanger {
            #[serde(rename = "npm-package-json")]
            #[serde(alias = "npm")]
            NpmPackageJson(npm_package_json::NpmPackageJson),
            #[serde(rename = "gradle-properties")]
            GradleProperties(gradle_properties::GradleProperties),
            #[serde(rename = "regex-pattern")]
            RegexPattern(regex_pattern::RegexPattern),
        }

        let repr: Reprs = Deserialize::deserialize(deserializer)?;

        Ok(match repr {
            InString(str) => parse_single_changer(&str),
            Tuple1((kind,)) => create_single_changer(&kind, None, None),
            Tuple2((kind, info)) => create_single_changer(&kind, Some(&info), None),
            Tuple3((kind, info, path)) => create_single_changer(&kind, Some(&info), Some(&path)),
            AsStruct(NpmPackageJson(changer)) => Box::new(changer),
            AsStruct(GradleProperties(changer)) => Box::new(changer),
            AsStruct(RegexPattern(changer)) => Box::new(changer),
        })
    }
}

pub(crate) fn parse_version_changers(parse: &str) -> VersionChangers {
    VersionChangers {
        changers: parse.split(';').map(parse_single_changer).collect(),
    }
}

fn parse_single_changer(parse: &str) -> Box<dyn DynVersionChanger> {
    let (kind, info, path) = if let Some((kind, rest)) = parse.split_once(':') {
        if let Some((info, path)) = rest.split_once('@') {
            (kind, info, path)
        } else {
            (kind, rest, "")
        }
    } else if let Some((kind, path)) = parse.split_once('@') {
        (kind, "", path)
    } else {
        (parse, "", "")
    };

    let info = if info.is_empty() { None } else { Some(info) };
    let path = if path.is_empty() { None } else { Some(path) };

    create_single_changer(kind, info, path)
}

fn create_single_changer(
    kind: &str,
    info: Option<&str>,
    path: Option<&str>,
) -> Box<dyn DynVersionChanger> {
    match kind {
        "npm" | "npm-package-json" => Box::new(npm_package_json::NpmPackageJson::parse(info, path)),
        "gradle-properties" => Box::new(gradle_properties::GradleProperties::parse(info, path)),
        "regex-pattern" => Box::new(regex_pattern::RegexPattern::parse(info, path)),
        unknown => panic!("unknown version changer kind: {}", unknown),
    }
}
