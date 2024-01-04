pub(crate) mod gradle;
pub(crate) mod json;
pub(crate) mod properties;

use std::io;
use std::path::Path;
use tokio::io::AsyncWriteExt;

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
