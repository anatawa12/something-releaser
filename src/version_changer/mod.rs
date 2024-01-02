mod npm_package_json;

use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

pub(crate) trait VersionChanger: Display {
    fn parse(parse: &str) -> Self;
    async fn load_version(&self) -> String;
    async fn set_version(&self, version: &str);
}

pub(crate) trait DynVersionChanger: Display {
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

pub(crate) fn parse_version_changers(parse: &str) -> VersionChangers {
    VersionChangers {
        changers: parse.split(';').map(parse_single_changer).collect(),
    }
}

fn parse_single_changer(parse: &str) -> Box<dyn DynVersionChanger> {
    let (kind, info) = parse.split_once(&[':', '@'][..]).unwrap_or((parse, ""));
    match kind {
        "npm" | "npm-package-json" => Box::new(npm_package_json::NpmPackageJson::parse(info)),
        unknown => panic!("unknown version changer kind: {}", unknown),
    }
}
