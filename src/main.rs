use clap::{AppSettings, Clap};
use log::{debug as verbose, error, info, trace, warn};

use ext::*;
use helpers::*;
pub use release_system::ReleaseSystem;

mod ext;
mod helpers;
mod release_system;

#[derive(Clap)]
pub struct CommonOptions {
    /// Show only warning or errors
    #[clap(short, long)]
    quiet: bool,
    /// Show verbose log
    #[clap(short, long)]
    verbose: bool,
    /// Debug verbose log
    #[clap(long)]
    debug: bool,
}

fn parse_common_options(options: &CommonOptions) {
    use stderrlog::*;
    let verbosity = if options.quiet {
        1 // warn
    } else if options.debug {
        4 // trace
    } else if options.verbose {
        3 // debug
    } else {
        2 // info
    };

    stderrlog::new()
        .module(module_path!())
        .verbosity(verbosity)
        .timestamp(Timestamp::Off)
        .init()
        .unwrap()
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
    ($($module: ident),* $(,)?) => {
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
            $module(commands::$module::Options)
            )*
        }
    };
}
____subcommands![actions,];
