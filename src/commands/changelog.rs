use std::cmp::Reverse;
use std::collections::HashSet;
use std::path::Path;

use chrono::{Date, DateTime, TimeZone, Utc};
use clap::Clap;
use git2::{Commit, Error, ObjectType, Oid, Reference, Repository, Time};
use html_escape::encode_text;
use regex::{Captures, Regex};
use url::Url;

use crate::*;

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
    let repo = Repository::open(&cwd).expect("failed to open cwd repository");
    info!("fetching tags...");
    let tags = fetch_tags(&repo, |x| option.filter.is_match(x)).await;
    info!("{} tags found", tags.len());
    let releases = tags
        .into_iter()
        .map(|x| parse_release(&repo, x))
        .collect::<Vec<_>>();
    let links: Box<dyn LinkCreator> = if let Some(url) = &option.github_repo_url {
        Box::new(GithubLinkCreator(url))
    } else {
        Box::new(SimpleLinkCreator)
    };
    create_markdown(releases, links.as_ref(), &mut std::io::stdout());
}

// instead of git2::Reference use this to clone
#[derive(Clone, Debug)]
struct Tag {
    name: String,
    target: Oid,
}

impl Tag {
    fn from_reference(reference: Reference) -> Option<Self> {
        let result = Self {
            name: reference.name()?.to_owned(),
            target: reference.target()?,
        };
        //verify: ref is targeting tag
        result.name.strip_prefix("refs/tags/")?;
        Some(result)
    }

    fn target(&self) -> Oid {
        return self.target;
    }

    fn name(&self) -> &str {
        return &self.name;
    }

    fn simple_name(&self) -> &str {
        return &self.name.strip_prefix("refs/tags/").unwrap();
    }
}

type TagCommit<'a> = (Tag, Commit<'a>);

async fn fetch_tags<F: Fn(&str) -> bool>(
    repo: &Repository,
    filter: F,
) -> Vec<(TagCommit<'_>, Option<TagCommit<'_>>)> {
    let tags = collect_tags(repo);
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
            return vec![];
        }
    };

    loop {
        let next = tags.next();
        result.push((cur, next.clone()));
        cur = match next {
            Some(v) => v,
            None => break,
        }
    }

    result.shrink_to_fit();

    result
}

fn collect_tags(repo: &Repository) -> Vec<String> {
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
    })
    .expect("collecting tags with tag_foreach");
    tags
}

//// get data about release

#[derive(Debug)]
struct ReleaseInfo<'r> {
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
    (new, prev): (TagCommit<'r>, Option<TagCommit<'r>>),
) -> ReleaseInfo<'r> {
    let prev = prev.map(|(t, c)| (Some(t), Some(c))).unwrap_or_default();
    let commits = get_commits(repo, prev.1.as_ref().map(|x| x.id()), new.1.id());

    if log::log_enabled!(log::Level::Trace) {
        trace!(
            "commits for {} (since {}, {}..{:?})",
            new.0.name(),
            prev.0.as_ref().map(|x| x.name()).unwrap_or("root"),
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

    let date = commits
        .first()
        .map(|x| x.committer().when())
        .and_then(|x| Utc.timestamp_opt(x.seconds(), 0).single());

    let is_empty_release = merges.is_empty() && fixes.is_empty();

    ReleaseInfo {
        merges,
        fixes,
        summary,
        date,
        tag: new.0,
        prev: prev.0,
        commits: if is_empty_release {
            prepare_commits(repo, commits)
        } else {
            vec![]
        },
    }
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
    commits.sort_by_key(|commit| {
        let commit_tree = commit.tree().ok();
        let parent_tree = commit.parent(0).ok().and_then(|x| x.tree().ok());
        let diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), commit_tree.as_ref(), None)
            .ok();
        let change_sum = diff
            .and_then(|diff| diff.stats().ok())
            .map(|stats| stats.insertions() + stats.deletions())
            .unwrap_or(0);
        Reverse(change_sum)
    });
    commits
}

fn get_commits(repo: &Repository, since: Option<Oid>, until: Oid) -> Vec<Commit> {
    let mut walk = repo.revwalk().expect("rev-walk init");
    walk.push(until).expect("setting rev walk push commit");
    if let Some(since) = since {
        walk.hide(since).expect("setting rev walk hide commit");
    }
    walk.into_iter()
        .filter_map(|x| x.map_err(|e| verbose!("error walking: {}", e)).ok())
        .filter_map(|x| {
            repo.find_commit(x)
                .map_err(|e| warn!("getting commit {}: {}", x, e))
                .ok()
        })
        .collect::<Vec<_>>()
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

trait LinkCreator {
    fn compare(&self, from: &str, to: &str) -> String;
    fn issue(&self, id: u32) -> String;
    fn merge(&self, id: u32) -> String;
    fn commit(&self, id: Oid) -> String;
}

struct SimpleLinkCreator;

impl LinkCreator for SimpleLinkCreator {
    fn compare(&self, _from: &str, to: &str) -> String {
        to.to_owned()
    }

    fn issue(&self, id: u32) -> String {
        format!("`#{}`", id)
    }

    fn merge(&self, id: u32) -> String {
        format!("`#{}`", id)
    }

    fn commit(&self, id: Oid) -> String {
        let instr = id.to_string();
        format!("`{}`", &instr[0..7])
    }
}

// &str: base url
struct GithubLinkCreator<'a>(&'a str);

impl<'a> LinkCreator for GithubLinkCreator<'a> {
    fn compare(&self, from: &str, to: &str) -> String {
        format!(
            "[{name}]({gh}/compare/{from}...{to})",
            name = to,
            gh = self.0,
            from = from,
            to = to,
        )
    }

    fn issue(&self, id: u32) -> String {
        format!("[`#{id}`]({gh}/issues/{id})", id = id, gh = self.0)
    }

    fn merge(&self, id: u32) -> String {
        format!("[`#{id}`]({gh}/pull/{id})", id = id, gh = self.0)
    }

    fn commit(&self, id: Oid) -> String {
        let short = &id.to_string()[0..7];
        format!(
            "[`{short}`]({gh}/commit/{id})",
            gh = self.0,
            id = id,
            short = short,
        )
    }
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
        if let Some(prev) = release.prev {
            writeln!(
                "#### {}",
                links.compare(prev.simple_name(), release.tag.simple_name())
            );
        } else {
            writeln!("#### {}", release.tag.simple_name());
        }
        writeln!();
        if let Some(time) = release.date {
            writeln!("> {}", time.date().format("%-d %B %Y"));
        }
        writeln!();
        for merge in release.merges {
            writeln!(
                "- {} {}",
                encode_text(&merge.message),
                links.merge(merge.id)
            );
        }
        for fix in release.fixes {
            write!("- {}", encode_text(fix.commit.summary().unwrap()));
            for id in fix.ids {
                write!(" {}", links.issue(id));
            }
            writeln!();
        }
        for commit in release.commits {
            writeln!(
                "- {} {}",
                encode_text(commit.summary().unwrap()),
                links.commit(commit.id())
            );
        }
        println!()
    }
    Ok(())
}
