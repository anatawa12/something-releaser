use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use serde::Deserialize;
use crate::utils::properties::PropertiesFile;
use crate::version_changer::VersionChanger;

#[derive(Debug, Deserialize)]
pub struct GradleProperties {
    #[serde(default = "path_default")]
    path: PathBuf,
    #[serde(alias="info", default = "default_property")]
    property: String,
}

fn path_default() -> PathBuf {
    PathBuf::from("gradle.properties")
}

fn default_property() -> String {
    "version".to_string()
}

impl Display for GradleProperties {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "gradle-properties(at {} prop {})", self.path.display(), self.property)
    }
}

impl VersionChanger for GradleProperties {
    fn parse(info: &str, path: &str) -> Self {
        Self {
            path: if path.is_empty() {
                path_default()
            } else {
                PathBuf::from(path)
            },
            property: if info.is_empty() {
                default_property()
            } else {
                info.to_string()
            },
        }
    }

    async fn load_version(&self) -> String {
        PropertiesFile::load_may_not_exist(&self.path)
            .await
            .expect("loading gradle.properties")
            .get(&self.property)
            .expect("getting version from gradle.properties")
    }

    async fn set_version(&self, version: &str) {
        let mut properties = PropertiesFile::load_may_not_exist(&self.path)
            .await
            .expect("loading gradle.properties");
        properties.set(&self.property, version.to_string());
        tokio::fs::write(&self.path, properties.to_string())
            .await
            .expect("writing gradle.properties");
    }
}
