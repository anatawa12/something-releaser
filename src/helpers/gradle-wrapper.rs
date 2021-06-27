use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Stdio;

use tokio::io;
use tokio::process::Command;

use super::*;

pub struct GradleWrapperHelper {
    cwd: PathBuf,
}

impl GradleWrapperHelper {
    pub fn new<P: Into<PathBuf>>(cwd: P) -> Self {
        Self { cwd: cwd.into() }
    }

    pub async fn run_tasks(
        &self,
        tasks: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> io::Result<()> {
        run_err(
            Command::new(self.cwd.join("gradlew"))
                .arg("--")
                .args(tasks)
                .stdin(Stdio::null())
                .current_dir(&self.cwd),
        )
        .await
    }
}
