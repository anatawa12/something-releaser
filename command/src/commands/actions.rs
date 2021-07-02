use std::path::Path;

use clap::Clap;
use git2::Repository;
use url::Url;

use crate::release_system::*;

use super::publish::run as publish_project;
use super::update_version::run as update_version;
use super::update_version_next::run as update_version_next;

pub async fn main(option: &Options) {
    let action = crate_releaser_action(&option.release_system);

    action.verify_exit();

    let cwd = std::env::current_dir().expect("failed to get cwd");
    let repo = Repository::open(&cwd).expect("failed to open git repository");

    println!("::group::changing version...");
    let info = update_version(
        &cwd,
        &repo,
        &action.version_changers.as_slice(),
        Path::new("CHANGELOG.md"),
        &option.repo.to_string(),
    )
    .await;
    println!("::endgroup::");

    println!("::group::publish");
    publish_project(&cwd, &action.publishers, &info, option.dry_run).await;
    println!("::endgroup::");

    let new_version = info.version.make_next_version();
    println!("::group::changing version for next: {}", new_version);
    update_version_next(
        &cwd,
        &repo,
        new_version,
        &action.version_changers.as_slice(),
    )
    .await;
    println!("::endgroup::");
}

/// Run processes for GitHub actions
///
/// 1. changes version name
/// 2. generates CHANGELOG.md
/// 3. commits and creates tag version and CHANGELOG.md changes
/// 4. build & publish
/// 5. changes & commits version name for next version (-SNAPSHOT suffixed)
/// 6. pushes
#[derive(Clap)]
#[clap(verbatim_doc_comment)]
pub struct Options {
    /// Repository to clone and upload.
    #[clap(long)]
    repo: Url,
    /// The release system to upgrade version, update version info.
    #[clap(short = 'r', long)]
    release_system: Vec<ReleaseSystem>,
    /// if this was specified, dry-runs publishing and pushing
    #[clap(long)]
    dry_run: bool,
}
