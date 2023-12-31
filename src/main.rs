use std::env::Args;
use std::process::exit;

fn main() {
    do_main(std::env::args())
}

fn do_main(mut args: Args) {
    match args.next().as_deref().map(|x| x.strip_suffix(".exe").unwrap_or(x)) {
        None => {
            eprintln!("No command specified");
            exit(1);
        }
        Some("something-releaser") => do_main(args),
        Some(e) => {
            eprintln!("unknown command: {e}");
            exit(1);
        }
    }
}
