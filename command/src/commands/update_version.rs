use std::path::{Path, PathBuf};

use clap::Clap;
use git2::Repository;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::spawn;
use url::Url;

use crate::*;
use crate::release_system::*;

use super::changelog::{ChangelogRepo, GithubLinkCreator};

pub async fn main(option: &Options) {
    let cwd = std::env::current_dir().expect("failed to get cwd");
    let repo = Repository::open(&cwd).expect("failed to open cwd git repo");

    let info = run(
        &cwd,
        &repo,
        option.version_changers.as_slice(),
        &option.changelog,
        &option.repo.to_string(),
    ).await;

    if is_actions_env() {
        info!("::set-output name=version::{}", info.version);
        info!("::set-output name=next_version::{}", info.version.make_next_version());
    }
    info!("new version name is {}", info.version);

    async fn write(path: &Path, body: &str) {
        File::create(path)
            .await
            .expect("creating")
            .write_all(body.as_bytes())
            .await
            .expect("writing error")
    }

    let mut handles = Vec::with_capacity(2);

    if let Some(html) = &option.version_release_note_html {
        let html = html.clone();
        let note = info.release_note_html.clone();
        handles.push(spawn(async move {
            write(&html, &note).await
        }));
    }
    if let Some(markdown) = &option.version_release_note_markdown {
        let markdown = markdown.clone();
        let note = info.release_note_markdown.clone();
        handles.push(spawn(async move {
            write(&markdown, &note).await
        }));
    }

    for x in handles.iter_mut() {
        x.await.expect("error");
    }
}

pub async fn run(
    target_dir: &Path,
    repo: &Repository,
    version_changers: &[&dyn VersionChanger],
    changelog: &Path,
    repo_url: &str,
) -> VersionInfo {
    let mut index = repo.index().expect("get index");

    // 1. change version name
    let (version, changed_files) = change_version(version_changers, &target_dir).await;
    repo.add_files(&mut index, changed_files.iter());
    let version_tag_name = format!("v{}", version);
    index.write().expect("write index");

    // 2. commit newer version
    repo.commit_head(&mut index, &version_tag_name);
    repo.reference(
        &format!("refs/tags/{}", version_tag_name),
        repo.head().expect("HEAD").target().unwrap(),
        false,
        &format!("crate tag {}", version_tag_name),
    ).expect_fn(|| format!("creating {}", version_tag_name));
    index.write().expect("write index");

    // 3. create changelog
    let (release_note_html, release_note_markdown) =
        create_changelog(target_dir, changelog, repo_url).await;
    repo.add_files(&mut index, [target_dir.join(changelog)].iter());

    repo.amend_commit_head(&mut index);
    repo.reference(
        &format!("refs/tags/{}", version_tag_name),
        repo.head().expect("HEAD").target().unwrap(),
        true,
        &format!("crate tag {}", version_tag_name),
    ).expect_fn(|| format!("creating {}", version_tag_name));
    index.write().expect("write index");

    VersionInfo {
        version,
        release_note_html,
        release_note_markdown,
    }
}

async fn change_version(
    changers: &[&dyn VersionChanger],
    path: &Path,
) -> (VersionName, Vec<PathBuf>) {
    let mut versions = vec![];
    let mut files_to_add = vec![];
    for x in changers {
        let group = start_group(format_args!("running changer {}", x.name()));
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
        drop(group);
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

async fn create_changelog(cwd: &Path, changelog: &Path, repo_url: &str) -> (String, String) {
    let repo = ChangelogRepo::open(cwd).expect("opening repo failed");
    let releases = repo
        .fetch_releases(tag_filter)
        .expect("fetching releases");
    let releases = repo.parse_releases(&releases).expect("parsing releases");
    let file = tokio::fs::File::create(cwd.join(changelog))
        .await
        .expect("create CHANGELOG");
    let mut file = tokio::io::BufWriter::new(file);
    repo.create_releases_markdown(
        &releases,
        &GithubLinkCreator::new(repo_url),
        &mut file,
    )
        .await
        .unwrap();
    file.flush().await.expect("write changelog");
    drop(file);

    let release_note_html = create_release_note_string!(buf, 
        repo.create_release_html(&releases[0], &GithubLinkCreator::new(repo_url), &mut buf)
            .await
            .unwrap(););
    let release_note_markdown = create_release_note_string!(buf,
        repo.create_release_markdown(&releases[0], &GithubLinkCreator::new(repo_url), &mut buf)
            .await
            .unwrap(););
    return (release_note_html, release_note_markdown);
}

// ^v?[\d.]+$
fn tag_filter(name: &str) -> bool {
    let name = name.trim_start_matches("v");
    for x in name.bytes() {
        if x == b'.' {} else if (b'0'..b'9').contains(&x) {} else {
            return false;
        }
    }
    true
}

/// Updates version information and commits them
#[derive(Clap)]
#[clap(verbatim_doc_comment)]
pub struct Options {
    /// Repository to clone and upload.
    #[clap(long)]
    repo: Url,
    /// The version changer
    #[clap(short = 'c', long, required = true)]
    version_changers: Vec<&'static dyn VersionChanger>,
    /// The path of CHANGELOG.md markdown file
    #[clap(long, default_value = "CHANGELOG.md")]
    changelog: PathBuf,
    /// The path to output release of a version in html
    #[clap(long)]
    version_release_note_html: Option<PathBuf>,
    /// The path to output release of a version in markdown
    #[clap(long)]
    version_release_note_markdown: Option<PathBuf>,
}
