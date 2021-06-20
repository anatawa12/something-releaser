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
        Self {
            cwd: cwd.into(),
        }
    }

    pub async fn is_initialized(&self) -> bool {
        tokio::fs::metadata(self.cwd.join(".git")).await.is_ok()
    }

    pub async fn init(&self) -> io::Result<()> {
        run_err(Command::new("git")
            .arg("init")
            .stdin(Stdio::null())
            .current_dir(&self.cwd))
            .await
    }

    pub async fn exists_remote(&self, remote: &str) -> bool {
        async fn internal(cwd: &Path, remote: &str) -> io::Result<bool> {
            Ok(Command::new("git")
                .arg("remote")
                .arg("get-url")
                .arg(remote)
                .current_dir(&cwd)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()?
                .wait()
                .await?
                .success())
        }
        internal(&self.cwd, remote).await.ok().unwrap_or(false)
    }

    pub async fn remote_delete(&self, remote: &str) -> io::Result<()> {
        run_err(Command::new("git")
            .arg("remote")
            .arg("remove")
            .arg(remote)
            .stdin(Stdio::null())
            .current_dir(&self.cwd))
            .await
    }

    pub async fn add_remote(&self, remote: &str, url: &str) -> io::Result<()> {
        run_err(Command::new("git")
            .arg("remote")
            .arg("add")
            .arg(remote)
            .arg(url)
            .stdin(Stdio::null())
            .current_dir(&self.cwd))
            .await
    }

    pub async fn fetch(&self, remote: &str) -> io::Result<()> {
        run_err(Command::new("git")
            .arg("fetch")
            .arg(remote)
            .stdin(Stdio::null())
            .current_dir(&self.cwd))
            .await?;
        run_err(Command::new("git")
            .arg("remote")
            .arg("set-head")
            .arg("--auto")
            .arg(remote)
            .stdin(Stdio::null())
            .current_dir(&self.cwd))
            .await?;
        Ok(())
    }

    pub async fn get_remote_head(&self, remote: &str) -> io::Result<String> {
        let ref_base = format!("refs/remotes/{}", remote);
        let ref_file_path = self.cwd
            .join(".git")
            .join(&ref_base)
            .join("HEAD");
        let ref_file = tokio::fs::read_to_string(ref_file_path).await?;

        if let Some(branch_name) = ref_file.trim().strip_prefix(&format!("ref: {}/", &ref_base)) {
            return Ok(branch_name.into())
        }
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a branch reference"))
    }

    pub async fn checkout_remote(&self, remote: &str, branch: &str) -> io::Result<()> {
        // checkout
        run_err(Command::new("git")
            .arg("checkout")
            .arg("-B")
            .arg(branch)
            .stdin(Stdio::null())
            .current_dir(&self.cwd))
            .await?;

        // reset to remote
        run_err(Command::new("git")
            .arg("reset")
            .arg("--hard")
            .arg(format!("refs/remotes/{}/{}", remote, branch))
            .stdin(Stdio::null())
            .current_dir(&self.cwd))
            .await?;

        // set upstream
        run_err(Command::new("git")
            .arg("branch")
            .arg("--set-upstream-to")
            .arg(format!("refs/remotes/{}/{}", remote, branch))
            .stdin(Stdio::null())
            .current_dir(&self.cwd))
            .await?;
        Ok(())
    }
}
