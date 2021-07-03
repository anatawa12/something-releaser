use std::path::{Path, PathBuf};

use clap::Clap;
use git2::Repository;

use crate::release_system::*;
use crate::*;

pub async fn main(option: &Options) {
    let cwd = std::env::current_dir().expect("failed to get cwd");
    let repo = Repository::open(&cwd).expect("failed to open cwd git repo");

    run(
        &cwd,
        &repo,
        option.new_version,
        &option.version_changers.as_slice(),
    )
    .await;
}

pub async fn run(
    target_dir: &Path,
    repo: &Repository,
    new_version: VersionName,
    version_changers: &[&dyn VersionChanger],
) {
    let mut index = repo.index().expect("get index");

    let changed_files =
        change_version_for_next(version_changers, new_version.of_snapshot(), &target_dir).await;
    repo.add_files(&mut index, changed_files.iter());
    let message = format!("prepare for next version: {}", new_version);
    repo.commit_head(&mut index, &message);
}

async fn change_version_for_next(
    changers: &[&dyn VersionChanger],
    version: VersionName,
    path: &Path,
) -> Vec<PathBuf> {
    let mut files_to_add = vec![];
    for x in changers {
        let paths = x
            .update_version_for_next(path, version)
            .await
            .expect_fn(|| format!("running {}", x.name()));
        files_to_add.extend_from_slice(&paths);
    }
    return files_to_add;
}

/// Updates version information and commits them
#[derive(Clap)]
#[clap(verbatim_doc_comment)]
pub struct Options {
    /// The name of next version.
    new_version: VersionName,
    /// The version changer
    #[clap(short = 'c', long, required = true)]
    version_changers: Vec<&'static dyn VersionChanger>,
}
