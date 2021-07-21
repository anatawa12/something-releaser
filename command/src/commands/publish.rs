use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use clap::Clap;

use crate::*;
use crate::release_system::command_builder::CommandBuilderMap;
use crate::release_system::*;

pub async fn main(option: &Options) {
    let cwd = std::env::current_dir().expect("failed to get cwd");

    let info = VersionInfo {
        version: option.version,
        release_note_html: read_to_string(&option.release_note_html)
            .expect("reading release note HTML"),
        release_note_markdown: read_to_string(&option.release_note_markdown)
            .expect("reading release note Markdown"),
    };

    run(&cwd, &option.publishers, &info, option.dry_run).await;
}

pub async fn run(
    project: &Path,
    publishers: &[&dyn Publisher],
    version_info: &VersionInfo,
    dry_run: bool,
) {
    let mut builders = CommandBuilderMap::new();
    for publisher in publishers {
        publisher.prepare_environment(version_info).await;
        publisher.publish_project(&mut builders).await;
    }
    for x in builders.values() {
        let name = x.name();
        let group = start_group(format_args!("running command {}", x.name()));
        let out = x
            .create_command_to_exec(dry_run)
            .current_dir(project)
            .env("RELEASE_NOTE_HTML", &version_info.release_note_html)
            .env("RELEASE_NOTE_MARKDOWN", &version_info.release_note_markdown)
            .spawn()
            .expect_fn(|| format!("running {}", name))
            .wait_with_output()
            .expect_fn(|| format!("running {}", name));
        if !out.status.success() {
            panic!("running {}", name)
        }
        drop(group);
    }
}

/// Builds project
#[derive(Clap)]
#[clap(verbatim_doc_comment)]
pub struct Options {
    /// The publisher to build
    #[clap(short = 'b', long)]
    publishers: Vec<&'static dyn Publisher>,
    /// The version name
    #[clap(long)]
    version: VersionName,
    /// The path to release note HTML
    #[clap(short = 'h', long)]
    release_note_html: PathBuf,
    /// The path to release note Markdown
    #[clap(short = 'm', long)]
    release_note_markdown: PathBuf,
    /// if this was specified, dry-runs publishing and pushing
    #[clap(long)]
    dry_run: bool,
}
