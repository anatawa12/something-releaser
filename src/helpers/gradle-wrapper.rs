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
        Self {
            cwd: cwd.into(),
            options: vec![],
        }
    }

    pub fn add_init_script(&mut self, path: impl AsRef<Path>) {
        self.options.push("--init-script".into());
        self.options.push(path.as_ref().as_os_str().to_owned());
    }

    pub fn add_property(&mut self, name: impl AsRef<str>, value: impl AsRef<str>) {
        self.options.push("--project-prop".into());
        self.options
            .push(format!("{}={}", name.as_ref(), value.as_ref()).into());
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
