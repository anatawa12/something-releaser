#[macro_use]
mod macros;
mod commands;
mod utils;
mod version_changer;
mod version_utilities;
mod github_actions_utilities;
mod env;

use crate::commands::gradle_intellij::GradleIntellij;
use crate::commands::gradle_maven::GradleMaven;
use crate::commands::gradle_plugin_portal::GradlePluginPortal;
use crate::commands::gradle_signing::GradleSigning;
use crate::commands::publish_to_curse_forge::PublishToCurseForge;
use crate::commands::send_discord::SendDiscord;
use crate::version_changer::VersionChangerCommand;
use clap::{Command, CommandFactory, Parser};
use std::num::NonZeroI32;
use std::process::exit;
use crate::github_actions_utilities::GithubActionsUtilities;
use crate::version_utilities::VersionUtilities;
use utils::MaybeStdin;

type CmdResult<T = ()> = Result<T, NonZeroI32>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    exit(match Frontend::parse().execute().await {
        Ok(()) => 0,
        Err(e) => e.get(),
    })
}

#[derive(Debug, Parser)]
#[command(multicall = true)]
enum Frontend {
    #[command(flatten)]
    Direct(Commands),
    #[clap(subcommand)]
    #[command(name = "something-releaser")]
    AsArgument(Commands),
}

impl Frontend {
    pub(crate) async fn execute(self) -> CmdResult {
        match self {
            Frontend::Direct(commands) => commands.execute().await,
            Frontend::AsArgument(commands) => commands.execute().await,
        }
    }
}

#[derive(Debug, Parser)]
enum Commands {
    // version utilities
    #[command(flatten)]
    VersionUtilities(VersionUtilities),

    // changer commands
    #[command(flatten)]
    VersionChangerCommand(VersionChangerCommand),

    // configure utilities
    PrepareGradleMaven(GradleMaven),
    PrepareGradleSigning(GradleSigning),
    PrepareGradlePluginPortal(GradlePluginPortal),
    PrepareGradleIntellij(GradleIntellij),

    // api utilities
    PublishToCurseForge(PublishToCurseForge),
    SendDiscord(SendDiscord),

    // github actions utils
    #[command(flatten)]
    GithubActionsUtilities(GithubActionsUtilities),

    // install commands: they are internal
    InternalList,
}

impl Commands {
    async fn execute(self) -> CmdResult {
        use Commands::*;
        match self {
            // version utilities
            VersionUtilities(cmd) => cmd.execute().await,

            // changer commands
            VersionChangerCommand(cmd) => cmd.execute().await,

            // configure utilities
            PrepareGradleMaven(cmd) => cmd.configure().await,
            PrepareGradleSigning(cmd) => cmd.configure().await,
            PrepareGradlePluginPortal(cmd) => cmd.configure().await,
            PrepareGradleIntellij(cmd) => cmd.configure().await,

            // api utilities
            PublishToCurseForge(cmd) => cmd.run().await,
            SendDiscord(cmd) => cmd.run().await,

            // github actions utils
            GithubActionsUtilities(cmd) => cmd.execute().await,

            // install commands: they are internal
            InternalList => {
                let command = Commands::command();
                command.get_subcommands()
                    .map(Command::get_name)
                    .filter(|x| !x.starts_with("internal-"))
                    .for_each(|x| println!("{}", x));

                ok!();
            }
        }
    }
}
