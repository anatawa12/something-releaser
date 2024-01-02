use crate::utils;
use crate::version_changer::VersionChanger;
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub(crate) struct NpmPackageJson {
    #[serde(default = "path_default")]
    path: PathBuf,
}

fn path_default() -> PathBuf {
    PathBuf::from("package.json")
}

impl Display for NpmPackageJson {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "npm(at {})", self.path.display())
    }
}

impl VersionChanger for NpmPackageJson {
    fn parse(info: &str, path: &str) -> Self {
        if !info.is_empty() {
            panic!("invalid npm package.json version changer");
        }
        Self {
            path: if path.is_empty() {
                path_default()
            } else {
                PathBuf::from(path)
            },
        }
    }

    async fn load_version(&self) -> String {
        let reader = &tokio::fs::read_to_string(&self.path)
            .await
            .expect("reading package.json");
        serde_json::from_str::<serde_json::Value>(reader)
            .expect("parsing package.json")
            .get("version")
            .expect("getting version from package.json")
            .as_str()
            .expect("version in package.json is not a string")
            .to_string()
    }

    async fn set_version(&self, version: &str) {
        let read = &tokio::fs::read_to_string(&self.path)
            .await
            .expect("reading package.json");
        let mut parsed = utils::json::parse_json(read).expect("parsing package.json");
        let as_object = parsed
            .value
            .as_object_mut()
            .expect("package.json is not an object");
        let quoted = utils::json::quote_string(version);
        as_object.set(r#""version""#, utils::json::Token::StringLiteral(&quoted));
        let created = parsed.to_string();
        tokio::fs::write(&self.path, created)
            .await
            .expect("writing package.json");
    }
}
