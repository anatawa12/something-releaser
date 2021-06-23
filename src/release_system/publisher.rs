use std::path::Path;

use crate::*;

#[async_trait()]
pub trait Publisher {
    async fn publish_project(&self, project: &Path, dry_run: bool) -> ();
    fn name(&self) -> &'static str;
}

types_enum!(Publisher {});
