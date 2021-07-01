use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use clap::Clap;

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

    run(&cwd, &option.builders, &info, false).await;
}

pub async fn run(
    project: &Path,
    builders: &[&dyn Builder],
    version_info: &VersionInfo,
    print_group: bool,
) {
    for builder in builders {
        if print_group {
            println!("::group::running builder {}", builder.name());
        }
        builder.build_project(&project, version_info).await;
        if print_group {
            println!("::endgroup::");
        }
    }
}

/// Builds project
#[derive(Clap)]
#[clap(verbatim_doc_comment)]
pub struct Options {
    /// The build system to build
    #[clap(short = 'b', long)]
    builders: Vec<&'static dyn Builder>,
    /// The version name
    #[clap(short = 'v', long)]
    version: VersionName,
    /// The path to release note HTML
    #[clap(short = 'h', long)]
    release_note_html: PathBuf,
    /// The path to release note Markdown
    #[clap(short = 'm', long)]
    release_note_markdown: PathBuf,
}
