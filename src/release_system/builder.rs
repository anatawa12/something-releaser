use std::path::Path;

use crate::release_system::VersionInfo;
use crate::*;

#[async_trait()]
pub trait Builder {
    async fn build_project(&self, project: &Path, version_info: &VersionInfo) -> ();
    fn name(&self) -> &'static str;
}

types_enum!(Builder {});
