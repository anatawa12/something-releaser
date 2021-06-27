use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::fs::{read_to_string, OpenOptions};
use tokio::io::{AsyncWriteExt, Result};

use crate::release_system::VersionName;
use crate::*;

type Paths = Vec<PathBuf>;

#[async_trait()]
pub trait VersionChanger {
    async fn is_updatable(&self, dir: &Path) -> bool;
    // returns: version name and changed files
    async fn update_version_for_release(&self, dir: &Path) -> Result<(VersionName, Paths)>;
    // returns: next version name and changed files
    async fn update_version_for_next(&self, dir: &Path, version: VersionName) -> Result<Paths>;
    fn name(&self) -> &'static str;
}

pub(super) struct GradlePropertiesVersionChanger;

async fn replace_file(
    properties_path: &Path,
    f: impl FnOnce(VersionName) -> VersionName,
) -> Result<VersionName> {
    let properties = read_to_string(&properties_path).await?;
    let mut properties_file = PropertiesFile::parse(&properties)?;
    verbose!("read properties file");

    let version = properties_file
        .find_value("version")
        .expect("version not found in gradle.properties");
    let version: VersionName = version.parse().expect("invalid version name");
    verbose!("current version: {}", version);
    let next_version = f(version);
    properties_file.set_value("version".to_owned(), next_version.to_string());
    verbose!("rewrite version: {}", next_version);

    let properties_size = fs::metadata(&properties_path).await.unwrap().len();
    verbose!("got current file size: {}", properties_size);
    // -snapshot: 9
    let mut buf = Vec::<u8>::with_capacity(properties_size as usize + 10);
    properties_file.write(&mut buf).expect("");
    verbose!("buffered");
    let mut properties = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&properties_path)
        .await?;
    properties.write_all(&buf).await?;
    properties.flush().await?;
    verbose!("flushed");
    Ok(next_version)
}

#[async_trait()]
impl VersionChanger for GradlePropertiesVersionChanger {
    async fn is_updatable(&self, dir: &Path) -> bool {
        async fn inner(dir: &Path) -> Option<()> {
            let properties = read_to_string(dir.join("gradle.properties")).await.ok()?;
            let properties = PropertiesFile::parse(&properties).ok()?;

            let version = properties.find_value("version")?;
            let _: VersionName = version.parse().ok()?;
            Some(())
        }
        inner(dir).await.is_some()
    }

    async fn update_version_for_release(&self, dir: &Path) -> Result<(VersionName, Paths)> {
        let properties_path = dir.join("gradle.properties");
        let next_version = replace_file(&properties_path, |version| {
            if version.snapshot() {
                version.un_snapshot()
            } else {
                warn!(
                    "current version is not a snapshot version, this will use next patch version!"
                );
                version.make_next_version()
            }
        })
        .await?;

        Ok((next_version, vec![properties_path]))
    }

    async fn update_version_for_next(&self, dir: &Path, version: VersionName) -> Result<Paths> {
        let properties_path = dir.join("gradle.properties");
        replace_file(&properties_path, |_| version).await?;
        Ok(vec![properties_path])
    }

    fn name(&self) -> &'static str {
        "gradle.properties version updater"
    }
}

types_enum!(VersionChanger {
    GradlePropertiesVersionChanger,
});
