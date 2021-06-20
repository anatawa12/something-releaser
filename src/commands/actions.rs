use std::path::Path;

use clap::Clap;
use url::Url;

use crate::*;

pub async fn main(option: &Options) {
    let origin = "origin";

    let cwd = std::env::current_dir().expect("failed to get cwd");

    let repo = init_git_repo(&cwd).await;
    set_remote(&repo, origin, &option.repo).await;
    clone_remote(&repo, origin, option.branch.as_ref().map(String::as_str)).await;
}

async fn init_git_repo(path: &Path) -> GitHelper {
    let helper = GitHelper::new(&path);
    if !helper.is_initialized().await {
        trace!("cwd looks not initialized");
        helper.init().await.expect("initialize failed");
    }
    helper
}

async fn set_remote(repo: &GitHelper, name: &str, remote: &Url) -> () {
    if repo.exists_remote(name).await {
        warn!("remote named '{}' found! This will override this!", name);
        repo.remote_delete(name)
            .await
            .expect_fn(|| format!("removing {} failed", name));
    }
    repo.add_remote(name, remote.as_str())
        .await
        .expect_fn(|| format!("adding {} failed!", name))
}

async fn clone_remote(repo: &GitHelper, remote: &str, branch: Option<&str>) {
    repo.fetch(remote)
        .await
        .expect_fn(|| format!("fetching {} failed", remote));
    let branch = if let Some(branch) = branch {
        branch.to_owned()
    } else {
        repo.get_remote_head(remote).await.expect_fn(|| {
            format!(
                "getting default branch of {} failed. not a branch?",
                remote
            )
        })
    };

    repo.checkout_remote(remote, &branch)
        .await
        .expect_fn(|| format!("checking out {}/{}", remote, &branch));
}

#[derive(Clap)]
pub struct Options {
    /// Repository to clone and upload.
    #[clap(long)]
    repo: Url,
    /// Branch name to be cloned and pushed.
    branch: Option<String>,
    /// The release system to upgrade version, update version info.
    #[clap(long)]
    release_system: Vec<ReleaseSystem>,
}
