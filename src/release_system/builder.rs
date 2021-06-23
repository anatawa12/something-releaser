use std::path::Path;

use crate::*;

#[async_trait()]
pub trait Builder {
    async fn build_project(&self, project: &Path) -> ();
    fn name(&self) -> &'static str;
}

types_enum!(Builder {});
