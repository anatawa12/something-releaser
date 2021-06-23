use std::path::{Path, PathBuf};

use tokio::io::Result;

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

types_enum!(VersionChanger {});
