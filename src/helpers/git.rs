use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io;
use tokio::process::Command;

use super::*;

pub struct GitHelper {
    cwd: PathBuf,
}

impl GitHelper {
    pub fn new<P: Into<PathBuf>>(cwd: P) -> Self {
        Self { cwd: cwd.into() }
    }

    pub async fn add_files<'a, I: Iterator<Item = R>, R: AsRef<Path>>(
        &self,
        files: I,
    ) -> io::Result<()> {
        let mut command = Command::new("git");
        command.arg("add");
        command.arg("--");
        for x in files {
            command.arg(x.as_ref());
        }

        run_err(command.stdin(Stdio::null()).current_dir(&self.cwd)).await?;

        Ok(())
    }

    pub async fn commit(&self, message: &str) -> io::Result<()> {
        run_err(
            Command::new("git")
                .arg("commit")
                .arg("--message")
                .arg(message)
                .stdin(Stdio::null())
                .current_dir(&self.cwd),
        )
        .await
    }

    pub async fn commit_amend(&self) -> io::Result<()> {
        run_err(
            Command::new("git")
                .arg("commit")
                .arg("--amend")
                .arg("--no-edi")
                .stdin(Stdio::null())
                .current_dir(&self.cwd),
        )
        .await
    }

    pub async fn add_tag(&self, name: &str, target: &str) -> io::Result<()> {
        run_err(
            Command::new("git")
                .arg("tag")
                .arg(name)
                .arg(target)
                .stdin(Stdio::null())
                .current_dir(&self.cwd),
        )
        .await
    }

    pub async fn remove_tag(&self, name: &str) -> io::Result<()> {
        run_err(
            Command::new("git")
                .arg("tag")
                .arg("--delete")
                .arg(name)
                .stdin(Stdio::null())
                .current_dir(&self.cwd),
        )
        .await
    }

    pub async fn push(&self, remote: &str) -> io::Result<()> {
        run_err(
            Command::new("git")
                .arg("push")
                .arg(remote)
                .stdin(Stdio::null())
                .current_dir(&self.cwd),
        )
        .await
    }
}
