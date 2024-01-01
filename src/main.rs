use std::env::Args;
use std::process::exit;

#[tokio::main]
async fn main() {
    exit(do_main(std::env::args()).await)
}

fn sanitize_cmd(mut cmd: &str) -> &str {
    if cfg!(windows) {
        cmd = cmd.strip_suffix(".exe").unwrap_or(cmd)
    }
    if let Some(slash) = cmd.rfind('/') {
        cmd = &cmd[slash + 1..]
    }
    cmd
}

async fn do_main(mut args: Args) -> i32 {
    loop {
        return match args.next().as_deref().map(sanitize_cmd) {
            None => {
                eprintln!("No command specified");
                1
            }
            Some("something-releaser") => continue,
            Some(e) => {
                eprintln!("unknown command: {e}");
                1
            }
        }
    }
}
