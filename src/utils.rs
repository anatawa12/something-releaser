pub(crate) mod gradle;
pub(crate) mod json;
pub(crate) mod properties;

use std::env::Args;
use std::io;
use std::path::Path;
use std::str::FromStr;
use tokio::io::AsyncWriteExt;

pub trait ArgsExt {
    fn next_parsed_or<T: FromStr>(&mut self, default: T) -> T;
}

impl ArgsExt for Args {
    fn next_parsed_or<T: FromStr>(&mut self, default: T) -> T {
        self.next().and_then(|s| s.parse().ok()).unwrap_or(default)
    }
}

#[inline]
pub async fn write_to_new_file(path: impl AsRef<Path>, content: &[u8]) -> io::Result<()> {
    async fn inner(path: &Path, content: &[u8]) -> io::Result<()> {
        tokio::fs::create_dir_all(path.parent().unwrap()).await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await?;
        file.write_all(content).await?;
        file.flush().await?;
        Ok(())
    }

    inner(path.as_ref(), content).await
}
