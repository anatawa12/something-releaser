use std::env::Args;
use std::process::exit;

#[tokio::main]
async fn main() {
    exit(do_main(std::env::args()).await)
}

async fn do_main(mut args: Args) -> i32 {
    loop {
        return match args.next().as_deref().map(|x| x.strip_suffix(".exe").unwrap_or(x)) {
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
