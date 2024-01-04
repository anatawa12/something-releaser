use crate::version_changer::VersionChangers;
use serde::Deserialize;
use std::collections::HashMap;
use log::debug;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ConfigFile {
    #[serde(alias = "releaseChanger", default)]
    pub release_changer: Option<VersionChangers>,
    #[serde(rename = "target", default)]
    pub targets: HashMap<String, TargetConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TargetConfig {
    #[serde(alias = "releaseChanger", default)]
    pub release_changer: Option<VersionChangers>,
}

pub(crate) async fn env_file() -> ConfigFile {
    if let Some(json) = read_or_none(".something-releaser.json").await {
        debug!("parsing .something-releaser.json");
        return serde_json::from_str(&json).expect("parsing .something-releaser.json");
    }

    if let Some(toml) = read_or_none(".something-releaser.toml").await {
        debug!("parsing .something-releaser.toml");
        return toml::from_str(&toml).expect("parsing .something-releaser.toml");
    }

    debug!("no config file found");
    return Default::default();

    async fn read_or_none(path: &str) -> Option<String> {
        match tokio::fs::read_to_string(path).await {
            Ok(s) => Some(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => panic!("reading {}: {}", path, e),
        }
    }
}
