use clap::Clap;
use url::Url;
use crate::*;

pub async fn main(option: &Options) {
}

#[derive(Clap)]
pub struct Options {
    /// Repository to clone and upload.
    #[clap(long)]
    repo: Url,
    /// The release system to upgrade version, update version info.
    #[clap(long)]
    release_system: Vec<ReleaseSystem>,
}
