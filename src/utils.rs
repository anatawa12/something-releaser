pub(crate) mod gradle;
pub(crate) mod json;
pub(crate) mod properties;

use crate::CmdResult;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::IsTerminal;
use std::path::Path;
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

#[derive(Debug, Clone, Default)]
pub(crate) enum MaybeStdin<T> {
    #[default]
    DefaultStdin,
    Stdin,
    Value(T),
}

impl<T: Display> Display for MaybeStdin<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeStdin::DefaultStdin | MaybeStdin::Stdin => f.write_str("-"),
            MaybeStdin::Value(v) => Display::fmt(v, f),
        }
    }
}

impl<T: FromStr> FromStr for MaybeStdin<T> {
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(Self::Stdin)
        } else {
            Ok(Self::Value(s.parse()?))
        }
    }
}

impl<T> MaybeStdin<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    pub async fn get(self, kind: &str) -> CmdResult<T> {
        match self {
            Self::DefaultStdin if io::stdin().is_terminal() => {
                err!("No {} specified. if you actually want to pass from console, pass '-' as the version", kind);
            }
            Self::Stdin | MaybeStdin::DefaultStdin => match Self::read_stdin().await?.parse() {
                Ok(value) => Ok(value),
                Err(e) => {
                    err!("invalid {}: {}", kind, e);
                }
            },
            Self::Value(value) => Ok(value),
        }
    }

    async fn read_stdin() -> CmdResult<String> {
        let mut read = String::new();
        tokio::io::stdin()
            .read_to_string(&mut read)
            .await
            .expect("reading stdin");
        if read.ends_with('\n') {
            read.pop();
        }
        if read.ends_with('\r') {
            read.pop();
        }
        Ok(read)
    }
}
