use tokio::io;
use tokio::process::Command;

pub use git::GitHelper;
pub use properties::PropertiesFile;

mod git;
mod properties;

// helper utils

async fn run_err(cmd: &mut Command) -> io::Result<()> {
    let exit_status = cmd.spawn()?.wait().await?;
    if !exit_status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("process exited with non-zero value"),
        ));
    }
    Ok(())
}
