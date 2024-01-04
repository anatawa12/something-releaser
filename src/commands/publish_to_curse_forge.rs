use crate::CmdResult;
use clap::{Args, Parser, ValueEnum};
use reqwest::multipart::Part;
use serde::Serialize;
use std::num::NonZeroU64;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Parser)]
#[command(name = "publish-to-curse-forge")]
#[command(no_binary_name = true)]
pub(crate) struct PublishToCurseForge {
    // required options
    /// The path to the jar file to upload
    #[arg(short, long)]
    file: PathBuf,
    /// The API Token for curse forge
    #[arg(short, long)]
    token: String,
    /// The project id of the project to upload to
    #[arg(short = 'i', long)]
    project_id: u64,
    /// The release kind
    #[arg(long)]
    release_type: ReleaseType,

    // metadata options
    /// The parent file id
    #[arg(short = 'p', long)]
    parent_file_id: Option<NonZeroU64>,
    /// The display name of the file
    #[arg(short = 'n', long)]
    name: Option<String>,
    #[command(flatten)]
    changelog: ChangelogGroup,
    /// the document format of the changelog
    #[arg(long, default_value = "html")]
    changelog_type: ChangelogType,

    /// The game versions compatible with
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    game_versions: Vec<u64>,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct ChangelogGroup {
    /// The changelog text
    #[arg(long)]
    changelog: Option<String>,
    /// Path to the changelog file
    #[arg(long)]
    changelog_file: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum ReleaseType {
    Alpha,
    Beta,
    Release,
}

#[derive(ValueEnum, Debug, Copy, Clone, Default, Serialize)]
#[serde(rename_all = "lowercase")]
enum ChangelogType {
    #[default]
    Html,
    Text,
    Markdown,
}

#[derive(Serialize)]
struct CurseMetadata<'a> {
    changelog: String,
    changelog_type: ChangelogType,
    display_name: Option<&'a str>,
    parent_file_id: Option<NonZeroU64>,
    game_versions: &'a [u64],
    release_type: ReleaseType,
}

impl PublishToCurseForge {
    pub async fn run(&self) -> CmdResult {
        let in_file = fs::File::open(&self.file).await.expect("opening file");

        let client = reqwest::Client::new();
        let mut form = reqwest::multipart::Form::new();
        form = form.part("file", Part::stream(in_file));
        form = form.text(
            "metadata",
            serde_json::to_string(&self.metadata().await).unwrap(),
        );

        let response = client
            .post(format!(
                "https://minecraft.curseforge.com/api/projects/{}/upload-file",
                self.project_id
            ))
            .header("X-Api-Token", self.token.clone())
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let text = response.text().await.unwrap();
        println!("{}", text);
        if status.is_success() {
            ok!()
        } else {
            err!()
        }
    }

    async fn metadata(&self) -> CurseMetadata<'_> {
        let changelog = match (&self.changelog.changelog, &self.changelog.changelog_file) {
            (Some(changelog), None) => changelog.clone(),
            (None, Some(changelog_file)) => fs::read_to_string(changelog_file)
                .await
                .expect("reading changelog"),
            (None, None) => panic!("no changelog"),
            (Some(_), Some(_)) => panic!("both changelog and changelog_file"),
        };
        CurseMetadata {
            changelog,
            changelog_type: self.changelog_type,
            display_name: self.name.as_deref(),
            parent_file_id: self.parent_file_id,
            game_versions: &self.game_versions,
            release_type: self.release_type,
        }
    }
}
