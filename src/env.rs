use std::collections::HashMap;
use serde::Deserialize;
use crate::version_changer::VersionChangers;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ConfigFile {
    #[serde(rename = "releaseChanger", default)]
    pub release_changer: Option<VersionChangers>,
    #[serde(rename = "target", default)]
    pub targets: HashMap<String, TargetConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TargetConfig {
    #[serde(rename = "releaseChanger", default)]
    pub release_changer: Option<VersionChangers>,
}

pub(crate) async fn env_file() -> ConfigFile {
    if let Some(json) = read_or_none(".something-releaser.json").await {
        return serde_json::from_str(&json).expect("parsing .something-releaser.json");
    }

    return Default::default();

    async fn read_or_none(path: &str) -> Option<String> {
        match tokio::fs::read_to_string(path).await {
            Ok(s) => Some(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => panic!("reading {}: {}", path, e),
        }
    }
}
