use std::path::{Path, PathBuf};

use clap::Clap;
use url::Url;

use crate::release_system::*;
use crate::*;

use super::update_version::run as update_version;

pub async fn main(option: &Options) {
    let action = crate_releaser_action(&option.release_system);

    action.verify_exit();

    let cwd = std::env::current_dir().expect("failed to get cwd");
    let repo = GitHelper::new(&cwd);

    println!("::group::changing version...");
    let info = update_version(
        &cwd,
        &repo,
        &action,
        Path::new("CHANGELOG.md"),
        &option.repo.to_string(),
    )
    .await;
    println!("::endgroup::");

    println!("::group::build");
    build_project(&cwd, &action.builders, &info).await;
    println!("::endgroup::");

    println!("::group::publish");
    publish_project(&cwd, &action.publishers, &info, option.dry_run).await;
    println!("::endgroup::");

    let new_version = info.version.make_next_version().of_snapshot();
    println!("::group::changing version for next: {}", new_version);
    let changed_files = change_version_for_next(&action.version_changers, new_version, &cwd).await;
    repo.add_files(changed_files.iter())
        .await
        .expect("add version files");
    println!("::endgroup::");

    println!("::group::next version commit");
    repo.commit(&format!(
        "prepare for next version: {}",
        new_version.un_snapshot()
    ))
    .await
    .expect("next version commit");
    println!("::endgroup::");

    if option.dry_run {
        info!("dry run specified! no push")
    } else {
        println!("::group::push");
        repo.push("origin").await.expect("push");
        println!("::endgroup::");
    }
}

async fn build_project(project: &Path, builders: &[&dyn Builder], version_info: &VersionInfo) {
    for builder in builders {
        println!("::group::running builder {}", builder.name());
        builder.build_project(&project, version_info).await;
        println!("::endgroup::");
    }
}

async fn publish_project(
    project: &Path,
    builders: &[&dyn Publisher],
    version_info: &VersionInfo,
    dry_run: bool,
) {
    for builder in builders {
        println!("::group::running publisher {}", builder.name());
        builder
            .publish_project(&project, version_info, dry_run)
            .await;
        println!("::endgroup::");
    }
}

async fn change_version_for_next(
    changers: &[&dyn VersionChanger],
    version: VersionName,
    path: &Path,
) -> Vec<PathBuf> {
    let mut files_to_add = vec![];
    for x in changers {
        println!("::group::running changer {}", x.name());
        let paths = x
            .update_version_for_next(path, version)
            .await
            .expect_fn(|| format!("running {}", x.name()));
        files_to_add.extend_from_slice(&paths);
        println!("::endgroup::");
    }
    return files_to_add;
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
