use std::path::{Path, PathBuf};
use std::process::exit;

use clap::Clap;
use url::Url;

use crate::release_system::*;
use crate::*;

use super::changelog::{ChangelogRepo, GithubLinkCreator};

pub async fn main(option: &Options) {
    let origin = "origin";

    let action = crate_releaser_action(&option.release_system);

    verify_releaser_action(&action);

    let cwd = std::env::current_dir().expect("failed to get cwd");

    println!("::group::cloning repository...");
    let repo = init_git_repo(&cwd).await;
    set_remote(&repo, origin, &option.repo).await;
    clone_remote(&repo, origin, option.branch.as_ref().map(String::as_str)).await;
    println!("::endgroup::");

    println!("::group::changing version...");
    let (version, changed_files) = change_version(&action.version_changers, &cwd).await;
    repo.add_files(changed_files.iter())
        .await
        .expect("add version files");
    let version_name = version.to_string();
    println!("::endgroup::");

    println!("::group::version commit & tag");
    repo.commit(&version_name).await.expect("commit");
    repo.add_tag(&version_name, "HEAD").await.expect("tag");
    println!("::endgroup::");

    println!("::group::changelog");
    let (release_note_html, release_note_markdown) =
        create_changelog(&cwd, &option.repo.to_string()).await;
    repo.add_files([cwd.join("CHANGELOG.md")].iter())
        .await
        .expect("add changelog");
    println!("::endgroup::");

    let info = VersionInfo {
        version,
        release_note_html,
        release_note_markdown,
    };

    println!("::group::changelog amend commit & re-tag");
    repo.commit_amend().await.expect("commit --amend");
    repo.remove_tag(&version_name).await.expect("remove tag");
    repo.add_tag(&version_name, "HEAD").await.expect("tag");
    println!("::endgroup::");

    println!("::group::build");
    build_project(&cwd, &action.builders, &info).await;
    println!("::endgroup::");

    println!("::group::publish");
    publish_project(&cwd, &action.publishers, &info, option.dry_run).await;
    println!("::endgroup::");

    let new_version = version.make_next_patch().of_snapshot();
    println!("::group::changing version for next: {}", new_version);
    let changed_files = change_version_for_next(&action.version_changers, version, &cwd).await;
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

fn verify_releaser_action(action: &ReleaserAction) {
    let mut errors = false;
    if action.version_changers.is_empty() {
        error!("no version changing release system specified!");
        errors = true;
    }
    if errors {
        exit(-1);
    }
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
        repo.get_remote_head(remote)
            .await
            .expect_fn(|| format!("getting default branch of {} failed. not a branch?", remote))
    };

    repo.checkout_remote(remote, &branch)
        .await
        .expect_fn(|| format!("checking out {}/{}", remote, &branch));
}

async fn change_version(
    changers: &[&dyn VersionChanger],
    path: &Path,
) -> (VersionName, Vec<PathBuf>) {
    let mut versions = vec![];
    let mut files_to_add = vec![];
    for x in changers {
        println!("::group::running changer {}", x.name());
        let (version, paths) = x
            .update_version_for_release(path)
            .await
            .expect_fn(|| format!("running {}", x.name()));
        assert!(
            !version.snapshot(),
            "logic failure: updated version must not snapshot"
        );
        versions.push(version);
        files_to_add.extend_from_slice(&paths);
        println!("::endgroup::");
    }
    let versions = versions
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    if versions.is_empty() {
        panic!("logic failre: require one or more changers")
    } else if versions.len() > 1 {
        error!(
            "multiple versions found! using first one: {}",
            versions.iter().next().unwrap()
        )
    }
    return (versions.into_iter().next().unwrap(), files_to_add);
}

macro_rules! create_release_note_string {
    ($name: ident, $write: stmt;) => {{
        let mut $name = Vec::<u8>::new();
        {
            $write
        }
        String::from_utf8($name).expect("invalid utf8 sequence")
    }};
}

async fn create_changelog(cwd: &Path, repo_url: &str) -> (String, String) {
    let repo = ChangelogRepo::open(cwd).expect("opening repo failed");
    let releases = repo
        .fetch_releases(|x| lazy_regex::regex_is_match!(r#"^[\d.]+$"#, x))
        .expect("fetching releases");
    let releases = repo.parse_releases(&releases).expect("parsing releases");
    let file = tokio::fs::File::create(cwd)
        .await
        .expect("create CHANGELOG");
    repo.create_releases_markdown(
        &releases,
        &GithubLinkCreator::new(repo_url),
        &mut tokio::io::BufWriter::new(file),
    )
    .await
    .unwrap();

    let release_note_html = create_release_note_string!(buf, 
        repo.create_release_html(&releases[0], &GithubLinkCreator::new(repo_url), &mut buf)
            .await
            .unwrap(););
    let release_note_markdown = create_release_note_string!(buf,
        repo.create_release_html(&releases[0], &GithubLinkCreator::new(repo_url), &mut buf)
            .await
            .unwrap(););
    return (release_note_html, release_note_markdown);
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
/// 1. clones repository
/// 2. changes version name
/// 3. generates CHANGELOG.md
/// 4. commits and creates tag version and CHANGELOG.md changes
/// 5. build & publish
/// 6. changes & commits version name for next version (-SNAPSHOT suffixed)
/// 7. pushes
#[derive(Clap)]
#[clap(verbatim_doc_comment)]
pub struct Options {
    /// Repository to clone and upload.
    #[clap(long)]
    repo: Url,
    /// Branch name to be cloned and pushed.
    branch: Option<String>,
    /// The release system to upgrade version, update version info.
    #[clap(short = 'r', long)]
    release_system: Vec<ReleaseSystem>,
    /// if this was specified, dry-runs publishing and pushing
    #[clap(long)]
    dry_run: bool,
}
