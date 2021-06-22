use std::path::Path;

use tokio::io::Result;

use crate::*;

#[async_trait()]
pub trait VersionChanger {
    async fn is_updatable(&self, dir: &Path) -> bool;
    async fn update_version_for_release(&self, dir: &Path) -> Result<()>;
    async fn update_version_for_next(&self, dir: &Path) -> Result<()>;
}

types_enum!(VersionChanger {});
