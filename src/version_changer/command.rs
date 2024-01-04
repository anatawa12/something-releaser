use std::env;
use clap::Parser;
use crate::CmdResult;
use crate::env::env_file;
use crate::utils::MaybeStdin;
use crate::version_changer::{parse_version_changers, VersionChangers};

#[derive(Debug, Parser)]
struct ChangerCommand {
    #[arg(short, long)]
    target: Option<String>,
}

impl ChangerCommand {
    pub async fn get_changer(&self) -> CmdResult<VersionChangers> {
        let mut env = env_file().await;

        if let Some(name) = &self.target {
            Ok(env
                .targets
                .get_mut(name)
                .and_then(|x| x.release_changer.as_mut())
                .map(std::mem::take)
                .unwrap_or_else(|| {
                    let env_name = format!("RELEASE_CHANGER_{}", name.to_ascii_uppercase());
                    parse_version_changers(
                        &env::var(&env_name).unwrap_or_else(|_| {
                            panic!("environment variable {} not set", env_name)
                        }),
                    )
                }))
        } else {
            Ok(env.release_changer.unwrap_or_else(|| {
                parse_version_changers(
                    &env::var("RELEASE_CHANGER")
                        .expect("environment variable RELEASE_CHANGER not set"),
                )
            }))
        }
    }
}

#[derive(Debug, Parser)]
#[allow(private_interfaces)]
pub enum VersionChangerCommand {
    GetVersion(ChangerCommand),
    SetVersion {
        #[command(flatten)]
        changer: ChangerCommand,
        #[arg(default_value_t = Default::default())]
        version: MaybeStdin<String>,
    },
}

impl VersionChangerCommand {
    pub async fn execute(self) -> CmdResult {
        use VersionChangerCommand::*;
        match self {
            GetVersion(changer) => {
                println!("{}", changer.get_changer().await?.get_version().await);
                ok!()
            }
            SetVersion { changer, version } => {
                changer
                    .get_changer()
                    .await?
                    .set_version(version.get("version").await?)
                    .await;
                ok!()
            }
        }
    }
}
