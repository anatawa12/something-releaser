use std::cmp::Reverse;
use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};
use clap::Clap;
use git2::{Commit, Oid, Reference, Repository};
use html_escape::encode_text;
use regex::{Captures, Regex};

use crate::*;

type GitResult<T> = Result<T, git2::Error>;

/// generates changelog like auto-changelog
#[derive(Clap)]
pub struct Options {
    #[clap(long, default_value = ".*")]
    filter: Regex,
    #[clap(long)]
    github_repo_url: Option<String>,
}

/// a command like auto-changelog
pub async fn main(option: &Options) {
    let cwd = std::env::current_dir().expect("failed to get cwd");
    let repo = ChangelogRepo::open(&cwd).expect("failed to open cwd repository");
    info!("fetching tags...");
    let releases = repo
        .fetch_releases(|x| option.filter.is_match(x))
        .expect("fetching release");
    info!("{} tags found", releases.len());
    let releases = repo
        .parse_releases(releases.iter())
        .expect("parsing release");
    let links: Box<dyn LinkCreator> = if let Some(url) = &option.github_repo_url {
        Box::new(GithubLinkCreator::new(url))
    } else {
        Box::new(SimpleLinkCreator)
    };
    repo.create_releases_markdown(releases, links.as_ref(), &mut std::io::stdout())
        .expect("writing markdown");
}

pub struct ChangelogRepo {
    repo: Repository,
}

pub struct ChangelogRelease<'r>(TagCommit<'r>, Option<TagCommit<'r>>);

#[allow(dead_code)]
impl ChangelogRepo {
    pub fn open(repo: &Path) -> GitResult<Self> {
        Ok(Self {
            repo: Repository::open(repo)?,
        })
    }

    pub fn fetch_releases<F: Fn(&str) -> bool>(
        &self,
        filter: F,
    ) -> GitResult<Vec<ChangelogRelease<'_>>> {
        fetch_tags(&self.repo, filter)
    }

    pub fn parse_releases<'a, 'r: 'a, I: IntoIterator<Item = &'a ChangelogRelease<'r>>>(
        &'r self,
        releases: I,
    ) -> GitResult<Vec<ReleaseInfo<'r>>> {
        return releases
            .into_iter()
            .map(|x| parse_release(&self.repo, x))
            .try_collect();
    }

    pub fn create_releases_markdown<W: std::io::Write>(
        &self,
        releases: Vec<ReleaseInfo>,
        links: &dyn LinkCreator,
        out: &mut W,
    ) -> std::io::Result<()> {
        create_markdown(releases, links, out)
    }

    pub fn create_release_markdown<W: std::io::Write>(
        &self,
        release: &ReleaseInfo,
        links: &dyn LinkCreator,
        out: &mut W,
    ) -> std::io::Result<()> {
        create_markdown_for_release(release, links, out)
    }

    pub fn create_release_html<W: std::io::Write>(
        &self,
        release: &ReleaseInfo,
        links: &dyn LinkCreator,
        out: &mut W,
    ) -> std::io::Result<()> {
        create_html_for_release(release, links, out)
    }
}

pub trait LinkCreator {
    fn compare_link(&self, from: &str, to: &str) -> Option<String>;
    fn issue_link(&self, id: u32) -> Option<String>;
    fn merge_link(&self, id: u32) -> Option<String>;
    fn commit_link(&self, id: Oid) -> Option<String>;
}

pub struct SimpleLinkCreator;

impl LinkCreator for SimpleLinkCreator {
    fn compare_link(&self, _from: &str, _to: &str) -> Option<String> {
        None
    }

    fn issue_link(&self, _id: u32) -> Option<String> {
        None
    }

    fn merge_link(&self, _id: u32) -> Option<String> {
        None
    }

    fn commit_link(&self, _id: Oid) -> Option<String> {
        None
    }
}

// &str: base url
pub struct GithubLinkCreator<'a>(&'a str);

impl<'a> GithubLinkCreator<'a> {
    pub fn new(repo_url: &'a str) -> Self {
        Self(repo_url)
    }
}

impl<'a> LinkCreator for GithubLinkCreator<'a> {
    fn compare_link(&self, from: &str, to: &str) -> Option<String> {
        Some(format!(
            "{gh}/compare/{from}...{to}",
            gh = self.0,
            from = from,
            to = to,
        ))
    }

    fn issue_link(&self, id: u32) -> Option<String> {
        Some(format!("{gh}/issues/{id}", id = id, gh = self.0))
    }

    fn merge_link(&self, id: u32) -> Option<String> {
        Some(format!("{gh}/pull/{id}", id = id, gh = self.0))
    }

    fn commit_link(&self, id: Oid) -> Option<String> {
        Some(format!("{gh}/commit/{id}", gh = self.0, id = id,))
    }
}

// instead of git2::Reference use this to clone
#[derive(Clone, Debug)]
struct Tag {
    simple_name: String,
    target: Oid,
}

impl Tag {
    fn from_reference(reference: Reference) -> Option<Self> {
        Some(Self {
            simple_name: result.name.strip_prefix("refs/tags/")?,
            target: reference.target()?,
        })
    }

    fn target(&self) -> Oid {
        return self.target;
    }

    fn simple_name(&self) -> &str {
        &self.simple_name
    }
}

type TagCommit<'a> = (Tag, Commit<'a>);

fn fetch_tags<F: Fn(&str) -> bool>(
    repo: &Repository,
    filter: F,
) -> GitResult<Vec<ChangelogRelease<'_>>> {
    let tags = collect_tags(repo)?;
    let tags_count = tags.len();
    trace!("tags found: {}", tags_count);
    let mut tags = tags
        .into_iter()
        .map(|x| {
            trace!("oid: {}", &x);
            x
        })
        .filter_map(|ref_name| repo.find_reference(&ref_name).ok())
        .filter_map(|tag| Tag::from_reference(tag))
        .filter_map(|t| repo.find_commit(t.target()).ok().map(|c| (t, c)))
        .filter(|(tag, _)| filter(tag.simple_name()))
        .collect::<Vec<_>>();

    tags.sort_by_key(|x| Reverse(x.1.committer().when()));

    let mut tags = tags.into_iter();

    let mut result = Vec::with_capacity(tags_count);

    let mut cur = match tags.next() {
        Some(v) => v,
        None => {
            trace!("all tags omitted or no tags");
            return Ok(vec![]);
        }
    };

    loop {
        let next = tags.next();
        result.push(ChangelogRelease(cur, next.clone()));
        cur = match next {
            Some(v) => v,
            None => break,
        }
    }

    result.shrink_to_fit();

    Ok(result)
}

fn collect_tags(repo: &Repository) -> GitResult<Vec<String>> {
    let mut tags = vec![];
    repo.tag_foreach(|oid, name| {
        if let Ok(name) = std::str::from_utf8(name) {
            tags.push(name.to_owned());
        } else {
            trace!(
                "found tag at {} but not a valid name, skipped: {:?}",
                oid,
                name
            );
        }
        true
    })?;
    Ok(tags)
}

//// get data about release

#[derive(Debug)]
pub struct ReleaseInfo<'r> {
    merges: Vec<MergeInformation<'r>>,
    fixes: Vec<CommitFixInformation<'r>>,
    commits: Vec<Commit<'r>>,
    summary: Option<String>,
    date: Option<DateTime<Utc>>,
    tag: Tag,
    prev: Option<Tag>,
}

fn parse_release<'r>(
    repo: &'r Repository,
    ChangelogRelease(new, prev): &ChangelogRelease,
) -> GitResult<ReleaseInfo<'r>> {
    let prev = prev
        .as_ref()
        .map(|(t, c)| (Some(t), Some(c)))
        .unwrap_or_default();
    let commits = get_commits(repo, prev.1.as_ref().map(|x| x.id()), new.1.id())?;

    if log::log_enabled!(log::Level::Trace) {
        trace!(
            "commits for {} (since {}, {}..{:?})",
            new.0.simple_name(),
            prev.0.as_ref().map(|x| x.simple_name()).unwrap_or("root"),
            new.1.id(),
            prev.1.as_ref().map(|x| x.id()),
        );
        for x in commits.iter() {
            trace!("  #{}: {}", x.id(), x.summary().unwrap_or("#invalid"))
        }
    }

    let merges = commits
        .iter()
        .filter_map(|x| try_parse_merge(x))
        .collect::<Vec<_>>();
    let fixes = commits
        .iter()
        .filter_map(|x| try_parse_fix(x))
        .collect::<Vec<_>>();

    let summary = commits
        .first()
        .and_then(|x| x.message())
        .map(|x| x.trim().to_owned());

    let date = Utc
        .timestamp_opt(new.1.committer().when().seconds(), 0)
        .single();

    let is_empty_release = merges.is_empty() && fixes.is_empty();

    Ok(ReleaseInfo {
        merges,
        fixes,
        summary,
        date,
        tag: new.0.clone(),
        prev: prev.0.cloned(),
        commits: if is_empty_release {
            prepare_commits(repo, commits)
        } else {
            vec![]
        },
    })
}

fn prepare_commits<'r>(repo: &Repository, mut commits: Vec<Commit<'r>>) -> Vec<Commit<'r>> {
    commits = commits
        .into_iter()
        .filter(|commit| {
            commit
                .summary()
                .map(|x| {
                    let is_valid = !lazy_regex::regex_is_match!(
                        r#"^v?\d+\.\d+\.\d+(-[a-zA-Z0-9-.]+)?(\+[0-9a-zA-Z0-9.-]+)?"#,
                        x
                    );
                    trace!("filter commit: commit message {} is {}", x, is_valid);
                    is_valid
                })
                .unwrap_or_default()
        })
        .collect();
    let mut commits_with_sorting = commits
        .into_iter()
        .map(|commit| {
            let commit_tree = commit.tree().ok();
            let parent_tree = commit.parent(0).ok().and_then(|x| x.tree().ok());
            let diff = repo
                .diff_tree_to_tree(parent_tree.as_ref(), commit_tree.as_ref(), None)
                .ok();
            let change_sum = diff
                .and_then(|diff| diff.stats().ok())
                .map(|stats| stats.insertions() + stats.deletions())
                .unwrap_or(0);
            (Reverse(change_sum), commit)
        })
        .collect::<Vec<_>>();

    commits_with_sorting.sort_by_key(|x| x.0);

    commits_with_sorting.into_iter().map(|x| x.1).collect()
}

fn get_commits(repo: &Repository, since: Option<Oid>, until: Oid) -> GitResult<Vec<Commit>> {
    let mut walk = repo.revwalk()?;
    walk.push(until)?;
    if let Some(since) = since {
        walk.hide(since)?;
    }
    Ok(walk
        .into_iter()
        .filter_map(|x| x.map_err(|e| verbose!("error walking: {}", e)).ok())
        .filter_map(|x| {
            repo.find_commit(x)
                .map_err(|e| warn!("getting commit {}: {}", x, e))
                .ok()
        })
        .collect::<Vec<_>>())
}

#[derive(Debug)]
struct MergeInformation<'r> {
    id: u32,
    message: String,
    commit: Commit<'r>,
}

fn try_parse_merge<'r>(commit: &Commit<'r>) -> Option<MergeInformation<'r>> {
    if commit.parent_count() <= 1 {
        return None;
    }

    let message = commit.message()?;
    let (id, msg): (&str, &str) = if let Some((_, id, msg)) =
        lazy_regex::regex_captures!(r#"Merge pull request #(\d+) from .+\n\n(.+)"#, message)
    {
        (id, msg)
    } else if let Some((_, msg, id)) =
        lazy_regex::regex_captures!(r#"(.+) \(#(\d+)\)(?:$|\n\n)"#, message)
    {
        (id, msg)
    } else {
        return None;
    };
    let id: u32 = id.parse().ok()?;

    Some(MergeInformation {
        id,
        message: msg.to_owned(),
        commit: commit.clone(),
    })
}

#[derive(Debug)]
struct CommitFixInformation<'r> {
    ids: Vec<u32>,
    commit: Commit<'r>,
}

fn try_parse_fix<'r>(commit: &Commit<'r>) -> Option<CommitFixInformation<'r>> {
    let regex: &Regex = lazy_regex::regex!(
        r#"(?:close[sd]?|fixe?[sd]?|resolve[sd]?)\s(?:#(\d+)|(https?://.+?/(?:issues|pull|pull-requests|merge_requests)/(\d+)))"#
    );

    let mut ids = vec![];

    for x in regex.captures(commit.message()?) {
        if let Some(id) = try_parse_fix_single_capture(&x) {
            ids.push(id)
        }
    }

    if ids.is_empty() {
        return None;
    }

    Some(CommitFixInformation {
        ids,
        commit: commit.clone(),
    })
}

fn try_parse_fix_single_capture(captures: &Captures) -> Option<u32> {
    (0..captures.len())
        .find_map(|i| captures.get(i))?
        .as_str()
        .parse()
        .ok()
}

macro_rules! markdown_link {
    ($link: expr, $inner_format: expr $(,$elements: expr)* $(,)?) => {
        if let Some(link) = $link {
            format!(concat!("[", $inner_format, "]({})") $(,$elements)* , link)
        } else {
            format!($inner_format $(,$elements)*)
        }
    };
}

fn create_markdown<W: std::io::Write>(
    releases: Vec<ReleaseInfo>,
    links: &dyn LinkCreator,
    out: &mut W,
) -> std::io::Result<()> {
    macro_rules! writeln {
        ($($arg:tt)*) => {
            std::writeln!(out, $($arg)*)?
        };
    }
    #[allow(unused_macros)]
    macro_rules! write {
        ($($arg:tt)*) => {
            std::write!(out, $($arg)*)?
        };
    }
    writeln!("### Changelog");
    writeln!();
    writeln!("All notable changes to this project will be documented in this file. Dates are displayed in UTC.");
    writeln!();
    // TODO: add link after creating git repository
    writeln!("Generated by `something-releaser`.");
    writeln!();
    for release in releases {
        writeln!(
            "#### {}",
            markdown_link!(
                release
                    .prev
                    .as_ref()
                    .and_then(|x| links.compare_link(x.simple_name(), release.tag.simple_name())),
                "{}",
                release.tag.simple_name(),
            ),
        );
        if let Some(time) = release.date {
            writeln!("> {}", time.date().format("%-d %B %Y"));
            writeln!();
        }
        writeln!();
        create_markdown_for_release(&release, links, out)?;
        writeln!();
    }
    Ok(())
}

fn create_markdown_for_release<W: std::io::Write>(
    release: &ReleaseInfo,
    links: &dyn LinkCreator,
    out: &mut W,
) -> std::io::Result<()> {
    macro_rules! writeln {
        ($($arg:tt)*) => {
            std::writeln!(out, $($arg)*)?
        };
    }
    macro_rules! write {
        ($($arg:tt)*) => {
            std::write!(out, $($arg)*)?
        };
    }
    for merge in &release.merges {
        writeln!(
            "- {} {}",
            encode_text(&merge.message),
            markdown_link!(links.merge_link(merge.id), "`#{}`", merge.id),
        );
    }
    for fix in &release.fixes {
        write!("- {}", encode_text(fix.commit.summary().unwrap()));
        for id in &fix.ids {
            write!(" {}", markdown_link!(links.issue_link(*id), "`#{}`", *id));
        }
        writeln!();
    }
    for commit in &release.commits {
        let short = &commit.id().to_string()[0..7];
        writeln!(
            "- {} {}",
            encode_text(commit.summary().unwrap()),
            markdown_link!(links.commit_link(commit.id()), "`{}`", short),
        );
    }
    Ok(())
}

macro_rules! html_link {
    ($link: expr, $inner_format: expr $(,$elements: expr)* $(,)?) => {
        if let Some(link) = $link {
            format!(concat!("<a href=\"{}\">", $inner_format, "</a>") , encode_text(&link) $(,$elements)*)
        } else {
            format!($inner_format $(,$elements)*)
        }
    };
}

fn create_html_for_release<W: std::io::Write>(
    release: &ReleaseInfo,
    links: &dyn LinkCreator,
    out: &mut W,
) -> std::io::Result<()> {
    macro_rules! writeln {
        ($($arg:tt)*) => {
            std::writeln!(out, $($arg)*)?
        };
    }
    macro_rules! write {
        ($($arg:tt)*) => {
            std::write!(out, $($arg)*)?
        };
    }
    writeln!("<ul>");
    for merge in &release.merges {
        writeln!(
            "<li>{} {}</li>",
            encode_text(&merge.message),
            html_link!(links.merge_link(merge.id), "<code>#{}</code>", merge.id),
        );
    }
    for fix in &release.fixes {
        write!("<li>{}", encode_text(fix.commit.summary().unwrap()));
        for id in &fix.ids {
            write!(
                " {}",
                html_link!(links.issue_link(*id), "<code>#{}</code>", *id)
            );
        }
        writeln!("</li>");
    }
    for commit in &release.commits {
        let short = &commit.id().to_string()[0..7];
        writeln!(
            "<li>{} {}</li>",
            encode_text(commit.summary().unwrap()),
            html_link!(links.commit_link(commit.id()), "<code>{}</code>", short),
        );
    }
    writeln!("</ul>");
    Ok(())
}
