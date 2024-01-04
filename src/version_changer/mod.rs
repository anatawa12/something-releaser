mod npm_package_json;

use crate::env::env_file;
use crate::{CmdResult, MaybeStdin};
use clap::Parser;
use serde::de::value::SeqAccessDeserializer;
use serde::de::SeqAccess;
use serde::{Deserialize, Deserializer};
use std::env;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Parser)]
struct ChangerCommand {
    #[arg(short, long)]
    target: Option<String>,
}

impl ChangerCommand {
    pub async fn get_changer(&self) -> CmdResult<VersionChangers> {
        let mut env = env_file().await;

        if let Some(name) = &self.target {
            Ok(env
                .targets
                .get_mut(name)
                .and_then(|x| x.release_changer.as_mut())
                .map(std::mem::take)
                .unwrap_or_else(|| {
                    let env_name = format!("RELEASE_CHANGER_{}", name.to_ascii_uppercase());
                    parse_version_changers(
                        &env::var(&env_name).unwrap_or_else(|_| {
                            panic!("environment variable {} not set", env_name)
                        }),
                    )
                }))
        } else {
            Ok(env.release_changer.unwrap_or_else(|| {
                parse_version_changers(
                    &env::var("RELEASE_CHANGER")
                        .expect("environment variable RELEASE_CHANGER not set"),
                )
            }))
        }
    }
}

#[derive(Debug, Parser)]
#[allow(private_interfaces)]
pub enum VersionChangerCommand {
    GetVersion(ChangerCommand),
    SetVersion {
        #[command(flatten)]
        changer: ChangerCommand,
        #[arg(default_value_t = Default::default())]
        version: MaybeStdin<String>,
    },
}

impl VersionChangerCommand {
    pub async fn execute(self) -> CmdResult {
        use VersionChangerCommand::*;
        match self {
            GetVersion(changer) => {
                println!("{}", changer.get_changer().await?.get_version().await);
                ok!()
            }
            SetVersion { changer, version } => {
                changer
                    .get_changer()
                    .await?
                    .set_version(version.get("version").await?)
                    .await;
                ok!()
            }
        }
    }
}

pub(crate) trait VersionChanger: Display + Debug {
    fn parse(info: &str, path: &str) -> Self;
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
        }

        let repr: Reprs = Deserialize::deserialize(deserializer)?;

        Ok(match repr {
            InString(str) => parse_single_changer(&str),
            Tuple1((kind,)) => create_single_changer(&kind, "", ""),
            Tuple2((kind, info)) => create_single_changer(&kind, &info, ""),
            Tuple3((kind, info, path)) => create_single_changer(&kind, &info, &path),
            AsStruct(NpmPackageJson(changer)) => Box::new(changer),
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

    create_single_changer(kind, info, path)
}

fn create_single_changer(kind: &str, info: &str, path: &str) -> Box<dyn DynVersionChanger> {
    match kind {
        "npm" | "npm-package-json" => Box::new(npm_package_json::NpmPackageJson::parse(info, path)),
        unknown => panic!("unknown version changer kind: {}", unknown),
    }
}
