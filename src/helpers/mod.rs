use tokio::io;
use tokio::process::Command;

pub use git::GitHelper;
pub use gradle_wrapper::GradleWrapperHelper;
pub use properties::PropertiesFile;

use crate::*;

mod git;
#[path = "gradle-wrapper.rs"]
mod gradle_wrapper;
mod properties;

// helper utils

async fn run_err(cmd: &mut Command) -> io::Result<()> {
    trace!("run: {:?}", cmd);
    let exit_status = cmd.spawn()?.wait().await?;
    if !exit_status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("process exited with non-zero value"),
        ));
    }
    Ok(())
}
