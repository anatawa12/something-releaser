use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io;
use tokio::process::Command;

use super::*;

pub struct GradleWrapperHelper {
    cwd: PathBuf,
    options: Vec<OsString>,
}

impl GradleWrapperHelper {
    pub fn new<P: Into<PathBuf>>(cwd: P) -> Self {
        let mut options = vec![];
        if log::log_enabled!(log::Level::Trace) {
            options.push("--stacktrace".into());
        }
        Self {
            cwd: cwd.into(),
            options,
        }
    }

    pub fn add_init_script(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.options.push("--init-script".into());
        self.options.push(path.as_ref().as_os_str().to_owned());
        self
    }

    pub async fn run_tasks(
        &self,
        tasks: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> io::Result<()> {
        run_err(
            Command::new(self.cwd.join("gradlew"))
                .args(&self.options)
                .arg("--")
                .args(tasks)
                .stdin(Stdio::null())
                .current_dir(&self.cwd),
        )
        .await
    }
}
