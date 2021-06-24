use std::path::Path;

use crate::release_system::VersionInfo;
use crate::*;

#[async_trait()]
pub trait Publisher {
    async fn publish_project(
        &self,
        project: &Path,
        version_info: &VersionInfo,
        dry_run: bool,
    ) -> ();
    fn name(&self) -> &'static str;
}

types_enum!(Publisher {});
