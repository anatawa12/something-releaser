use std::path::{Path, PathBuf};

use clap::Clap;

use crate::release_system::*;

use crate::*;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::sealedbox::seal;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Formatter, Display};
use std::str::FromStr;
use serde_yaml::{Value, Mapping};

const ENV_NAME: &str = "deployment";

pub async fn main(option: &Options) {
    if is_actions_env() {
        panic!("actions env not supported")
    }

    let repo: RepositoryNamePair = dialoguer::Input::new()
        .with_prompt("your repository name")
        .interact()
        .expect("enter correct repository name");

    let pat = dialoguer::Password::new()
        .with_prompt("Github Personal Access Token")
        .interact()
        .expect("enter correct PAT");

    let publisher_indices = dialoguer::MultiSelect::new()
        .with_prompt("Select Publisher")
        .items(<dyn Publisher>::names())
        .interact()
        .expect("enter correct");

    let publisher_names = publisher_indices
        .iter()
        .map(|x| <dyn Publisher>::names()[*x])
        .collect::<Vec<_>>();

    let secret_names = publisher_indices
        .iter()
        .map(|x| <dyn Publisher>::values()[*x])
        .flat_map(|x| x.secrets())
        .map(|x| *x)
        .collect::<Vec<_>>();

    let client = Client::builder()
        .user_agent("something-releaser+set_environments/1")
        .build()
        .unwrap();

    let secrets = read_secrets(&option.secrets);
    make_environment(&client, &pat, &repo).await;
    let (key, key_id) = get_public_key(&client, &pat, &repo).await;

    let secrets = {
        let mut environments = HashMap::new();
        for x in secret_names {
            let v = secrets.get(x).expect_fn(|| format!("no {} in secrets", x));
            environments.insert(x, v);
        }
        environments
    };

    for (k, v) in &secrets {
        set_env_secret(&client, &pat, &repo, k, v, &key, &key_id).await;
    }

    print_generated_yaml(
        publisher_names, 
        secrets.keys(),
    );
}

fn read_secrets(path: &Path) -> HashMap<String, String> {
    let body = std::fs::read_to_string(path).expect("can't read");
    serde_yaml::from_str::<HashMap<String, String>>(&body)
        .expect("invalid yaml")
}

async fn make_environment(client: &Client, pat: &str, repo: &RepositoryNamePair) {
    let res = client
        .get(format!(
            "https://api.github.com/repos/{}/environments/{}",
            repo, ENV_NAME
        ))
        .basic_auth("", Some(pat))
        .send()
        .await
        .expect("getting environment");
    match res.status() {
        StatusCode::OK => {
            info!("environment {} found!", ENV_NAME);
        }
        StatusCode::NOT_FOUND => {
            info!("environment {} not found. creating...", ENV_NAME);
            #[derive(Serialize)]
            struct PutEnvironments {
                wait_timer: u32,
            }
            client
                .put(format!(
                    "https://api.github.com/repos/{}/environments/{}",
                    repo, ENV_NAME
                ))
                .basic_auth("", Some(pat))
                .json(&PutEnvironments { wait_timer: 0 })
                .send()
                .await
                .expect("creating environment")
                .error_for_status()
                .expect("creating environment");
        }
        s => {
            panic!("unknown: {}", s);
        }
    }
}

async fn get_public_key(
    client: &Client,
    pat: &str,
    repo: &RepositoryNamePair,
) -> (PublicKey, String) {
    #[derive(Deserialize)]
    struct PublicKeyResponse {
        key: String,
        key_id: String,
    }
    let res = client
        .get(format!(
            "https://api.github.com/repos/{}/environments/{}/secrets/public-key",
            repo, ENV_NAME
        ))
        .basic_auth("", Some(pat))
        .send()
        .await
        .expect("getting environment public key")
        .error_for_status()
        .expect("getting environment public key")
        .json::<PublicKeyResponse>()
        .await
        .expect("getting environment public key");
    let key = PublicKey::from_slice(
        base64::decode(res.key)
            .expect("public key is not base64")
            .as_slice(),
    )
    .expect("public key is not valid");

    (key, res.key_id)
}

async fn set_env_secret(
    client: &Client,
    pat: &str,
    repo: &RepositoryNamePair,
    name: &str,
    value: &str,
    key: &PublicKey,
    key_id: &str,
) {
    #[derive(Serialize)]
    struct PutSecretRequest<'s> {
        encrypted_value: &'s str,
        key_id: &'s str,
    }
    client
        .put(format!(
            "https://api.github.com/repos/{}/environments/{}/secrets/{}",
            repo, ENV_NAME, name
        ))
        .basic_auth("", Some(pat))
        .json(&PutSecretRequest {
            encrypted_value: &base64::encode(seal(value.as_bytes(), &key)),
            key_id,
        })
        .send()
        .await
        .expect_fn(|| format!("setting secret {}", name))
        .error_for_status()
        .expect_fn(|| format!("setting secret {}", name));
}

macro_rules! y_map {
    (
        $(
             $k: expr => $v: expr
        ),*
        $(,)?
    ) => {
        {
            let mut map = Mapping::new();
            $(
            map.insert($k.into(), $v.into());
            )*
            Value::Mapping(map)
        }
    };
}

fn print_generated_yaml<'a, 'b>(
    publishers: impl IntoIterator<Item = impl Display>,
    secrets: impl IntoIterator<Item = impl IntoString + Display>,
) {
    let yaml = y_map! {
        "name" => "Publisher",
        "on" => Value::Sequence(vec!["workflow_dispatch".into()]),
        "jobs" => y_map! {
            "build" => y_map! {
                "environment" => "deployment",
                "runs-on" => "ubuntu-latest",
                "steps" => Value::Sequence(vec![
                    y_map!("uses" => "actions/checkout@v2"),
                    y_map!("uses" => "anatawa12/something-releaser/set_user@v1"),
                    y_map!{
                        "name" => "Set up JDK 1.8",
                        "uses" => "actions/setup-java@v1",
                        "with" => y_map!("java-version" => 1.8),
                    },
                    y_map!{
                        "uses" => "anatawa12/something-releaser/verup@v1",
                        "id" => "verup",
                        "with" => y_map!("version_changers" => "gradle-properties"),
                    },
                    y_map!{
                        "uses" => "anatawa12/something-releaser/publish@v1",
                        "with" => y_map!{
                            "publishers" => Value::String(publishers.into_iter().join(",")),
                            "changelog_html" => "${{ steps.verup.outputs.changelog_html }}",
                            "changelog_markdown" => "${{ steps.verup.outputs.changelog_markdown }}",
                            "version_name" => "${{ steps.verup.outputs.version }}",
                        },
                        "env" => {
                            let mut map = Mapping::new();
                            for x in secrets.into_iter() {
                                let v = format!("${{{{ secrets.{} }}}}", &x);
                                map.insert(Value::String(x.into_string()), Value::String(v));
                            }
                            Value::Mapping(map)
                        },
                    },
                    y_map!{
                        "uses" => "anatawa12/something-releaser/verup_next@v1",
                        "with" => y_map! {
                            "new_version" => "${{ steps.verup.outputs.new_version }}",
                            "version_changers" => "gradle-properties",
                        },
                    },
                    y_map!{
                        "name" => "Push",
                        "run" => "git push",
                    },
                ]),
            },
        }
    };

    serde_yaml::to_writer(&mut std::io::stdout(), &yaml).expect("writing yaml");
}

#[derive(Clone)]
struct RepositoryNamePair(String, String);
#[derive(Debug)]
struct RepositoryNamePairErr(&'static str);

impl FromStr for RepositoryNamePair {
    type Err = RepositoryNamePairErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (user, repo) = s.split_once('/').ok_or(RepositoryNamePairErr("no slash"))?;
        if repo.contains('/') {
            return Err(RepositoryNamePairErr("multiple slash"));
        }
        if user.contains(|x: char| !x.is_ascii_alphanumeric() && !"-_".contains(x)) {
            return Err(RepositoryNamePairErr("invalid char in user"));
        }
        if repo.contains(|x: char| !x.is_ascii_alphanumeric() && !"-_.".contains(x)) {
            return Err(RepositoryNamePairErr("invalid char in repo"));
        }
        Ok(RepositoryNamePair(user.to_owned(), repo.to_owned()))
    }
}

impl fmt::Display for RepositoryNamePair {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", &self.0, &self.1)
    }
}

impl fmt::Display for RepositoryNamePairErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// Sets GitHub Environment
#[derive(Clap)]
pub struct Options {
    /// the path to secrets.yaml
    secrets: PathBuf,
}
