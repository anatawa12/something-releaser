use std::fmt::{Debug, Display};
use std::path::Path;

use git2::{Index, Repository};
use rand::prelude::SliceRandom;
use rand::RngCore;

pub(crate) trait ResultExt<T, R> {
    fn expect_fn<S: Display, F: FnOnce() -> S>(self, func: F) -> T;
}

impl<T, R: Debug> ResultExt<T, R> for Result<T, R> {
    fn expect_fn<S: Display, F: FnOnce() -> S>(self, func: F) -> T {
        match self {
            Ok(t) => t,
            Err(e) => panic!("{}: {:?}", func(), e),
        }
    }
}

pub(crate) trait StrExt {
    fn escape_groovy(&self) -> String;
}

impl StrExt for str {
    fn escape_groovy(&self) -> String {
        let mut builder = Vec::<u8>::with_capacity(self.len());
        for x in self.bytes() {
            if x == b'\r' {
                builder.extend_from_slice(b"\\r")
            } else if x == b'\n' {
                builder.extend_from_slice(b"\\n")
            } else if x == b'\\' {
                builder.extend_from_slice(b"\\\\")
            } else if x == b'$' {
                builder.extend_from_slice(b"\\$")
            } else if x == b'\'' {
                builder.extend_from_slice(b"\\\'")
            } else if x == b'\"' {
                builder.extend_from_slice(b"\\\"")
            } else {
                builder.push(x)
            }
        }
        unsafe { String::from_utf8_unchecked(builder) }
    }
}

impl StrExt for String {
    fn escape_groovy(&self) -> String {
        self.as_str().escape_groovy()
    }
}

pub(crate) trait RngExt {
    fn gen_ascii_rand(&mut self, len: usize) -> String;
}

const BASE_STR: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

impl<T: RngCore> RngExt for T {
    fn gen_ascii_rand(&mut self, len: usize) -> String {
        unsafe {
            String::from_utf8_unchecked(
                BASE_STR
                    .as_bytes()
                    .choose_multiple(self, len)
                    .cloned()
                    .collect(),
            )
        }
    }
}

pub(crate) trait RepositoryExt {
    fn commit_head(&self, index: &mut Index, message: &str, amend: bool);
    fn add_files(&self, index: &mut Index, files: impl Iterator<Item = impl AsRef<Path>>);
}

impl RepositoryExt for Repository {
    fn commit_head(&self, index: &mut Index, message: &str, amend: bool) {
        let signature = self.signature().expect("getting signature");
        let tree = self
            .find_tree(index.write_tree().expect("writing index as a tree"))
            .expect("tree not found");

        let head = self
            .head()
            .ok()
            .map(|x| self.find_commit(x.target().unwrap()).unwrap());

        let (message, parents) = if amend {
            let head = head.as_ref().unwrap();
            (
                if message.is_empty() {
                    head.message().unwrap()
                } else {
                    message
                },
                head.parents().collect(),
            )
        } else {
            if let Some(head) = head {
                (message, vec![head])
            } else {
                (message, vec![])
            }
        };
        let parents: Vec<_> = parents.iter().collect();

        self.commit(
            None,
            &signature,
            &signature,
            message,
            &tree,
            parents.as_slice(),
        )
        .expect("creating commit");
    }

    fn add_files(&self, index: &mut Index, files: impl Iterator<Item = impl AsRef<Path>>) {
        let repo_root = self.path().parent().unwrap();
        for x in files {
            let mut x = x.as_ref();
            if x.is_absolute() {
                x = x
                    .strip_prefix(&repo_root)
                    .expect_fn(|| format!("relativize {}", x.display()))
            }
            index
                .add_path(x)
                .expect_fn(|| format!("adding {}", x.display()))
        }
    }
}
