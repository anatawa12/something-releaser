use std::fmt::{Debug, Display};
use std::path::Path;

use git2::{Index, Repository};
use rand::prelude::SliceRandom;
use rand::RngCore;

pub(crate) trait ResultExt<T> {
    fn expect_fn<S: Display, F: FnOnce() -> S>(self, func: F) -> T;
}

impl<T, R: Debug> ResultExt<T> for Result<T, R> {
    fn expect_fn<S: Display, F: FnOnce() -> S>(self, func: F) -> T {
        match self {
            Ok(t) => t,
            Err(e) => panic!("{}: {:?}", func(), e),
        }
    }
}

impl<T> ResultExt<T> for Option<T> {
    fn expect_fn<S: Display, F: FnOnce() -> S>(self, func: F) -> T {
        match self {
            Some(t) => t,
            None => panic!("{}", func()),
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
    fn amend_commit_head(&self, index: &mut Index);
    fn commit_head(&self, index: &mut Index, message: &str);
    fn add_files(&self, index: &mut Index, files: impl Iterator<Item = impl AsRef<Path>>);
}

impl RepositoryExt for Repository {
    fn amend_commit_head(&self, index: &mut Index) {
        let tree = self
            .find_tree(index.write_tree().expect("writing index as a tree"))
            .expect("tree not found");

        let head = self
            .head()
            .ok()
            .map(|x| self.find_commit(x.target().unwrap()).unwrap());

        let hash = head
            .unwrap()
            .amend(None, None, None, None, None, Some(&tree))
            .expect("creating commit");
        self.head()
            .unwrap()
            .set_target(hash, "amend commit")
            .expect("setting head");
    }

    fn commit_head(&self, index: &mut Index, message: &str) {
        let signature = self.signature().expect("getting signature");
        let tree = self
            .find_tree(index.write_tree().expect("writing index as a tree"))
            .expect("tree not found");

        let head = self
            .head()
            .ok()
            .map(|x| self.find_commit(x.target().unwrap()).unwrap());

        let parents = head.as_ref().map(|x| vec![x]).unwrap_or_default();

        self.commit(
            Some("HEAD"),
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

pub trait IntoString {
    fn into_string(self) -> String;
}

impl IntoString for String {
    fn into_string(self) -> String {
        self
    }
}

impl IntoString for &str {
    fn into_string(self) -> String {
        self.to_owned()
    }
}

impl<'a, T: ?Sized> IntoString for &&'a T
where
    &'a T: IntoString,
{
    fn into_string(self) -> String {
        (*self).into_string()
    }
}
