use async_trait::async_trait;
use clap::{AppSettings, Clap};
use itertools::Itertools;
use log::{debug as verbose, error, info, trace, warn};

use ext::*;
use helpers::*;
pub use release_system::ReleaseSystem;

include!("macros.rs");
mod ext;
mod helpers;
mod release_system;
mod logger;

#[derive(Clap)]
pub struct CommonOptions {
    /// Show only warning or errors
    #[clap(short, long, global = true)]
    quiet: bool,
    /// Show verbose log
    #[clap(short, long, global = true)]
    verbose: bool,
    /// Debug verbose log
    #[clap(long, global = true)]
    debug: bool,
    /// Enable Github Actions Mode
    #[clap(long, global = true)]
    github_actions_mode: bool,
}

fn parse_common_options(options: &CommonOptions) {
    if options.github_actions_mode {
        logger::init_actions(module_path!());
    } else {
        logger::init_command_logger_with_options(options, module_path!());
    }
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColorAuto)]
pub struct RootOptions {
    #[clap(flatten)]
    common: CommonOptions,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[tokio::main]
async fn main() {
    do_main().await;
}

macro_rules! ____subcommands {
    ($($module: ident);* $(;)?) => {
        async fn do_main() {
            let opts: RootOptions = RootOptions::parse();

            parse_common_options(&opts.common);

            match opts.subcmd {
                $(
                SubCommand::$module(ref options) =>
                    commands::$module::main(options).await,
                )*
            }
        }

        mod commands {
            $(
            pub(crate) mod $module;
            )*
        }

        #[allow(non_camel_case_types)]
        #[derive(Clap)]
        enum SubCommand {
            $(
            $module(commands::$module::Options),
            )*
        }
    };
}
____subcommands![
    actions;
    changelog;
    update_version;
    update_version_next;
    publish;
];
