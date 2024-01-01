#[macro_use]
mod macros;

use std::env::Args;
use std::num::NonZeroI32;
use std::process::exit;

#[tokio::main]
async fn main() {
    exit(match do_main(std::env::args()).await {
        Ok(()) => 0,
        Err(e) => e.get(),
    })
}

type CmdResult<T = ()> = Result<T, NonZeroI32>;

fn sanitize_cmd(mut cmd: &str) -> &str {
    if cfg!(windows) {
        cmd = cmd.strip_suffix(".exe").unwrap_or(cmd)
    }
    if let Some(slash) = cmd.rfind('/') {
        cmd = &cmd[slash + 1..]
    }
    cmd
}

async fn do_main(mut args: Args) -> CmdResult<()> {
    loop {
        match args.next().as_deref().map(sanitize_cmd) {
            None => err!("No command specified"),
            Some("something-releaser") => continue,
            Some(other) => err!("unknown command: {other}"),
        }
    }
}
