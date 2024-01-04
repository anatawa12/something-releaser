use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use regex::{Regex, Replacer};
use serde::de::Error;
use serde::Deserialize;
use crate::version_changer::VersionChanger;

#[derive(Debug, Deserialize)]
pub struct RegexPattern {
    path: PathBuf,
    #[serde(alias = "info", deserialize_with = "deserialize_regex")]
    pattern: Regex,
}

fn deserialize_regex<'de, D>(de: D) -> Result<Regex, D::Error> where D: serde::de::Deserializer<'de> {
    struct Visitor;
    
    impl serde::de::Visitor<'_> for Visitor {
        type Value = Regex;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a regex with $1")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
            str_to_regex(v).map_err(Error::custom)
        }
    }
    
    de.deserialize_str(Visitor)
}

fn str_to_regex(s: &str) -> Result<Regex, regex::Error> {
    let Some((before, after)) = s.split_once("$1") else {
        return Err(regex::Error::Syntax("missing $1".to_string()))
    };

    let _ = Regex::new(before)?;
    let _ = Regex::new(after)?;

    Regex::new(&format!("(?<prefix>{before})(?<version>.*)(?<suffix>{after})"))
}

impl Display for RegexPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "regex-pattern(at {} with {})", self.path.display(), self.pattern)
    }
}

impl VersionChanger for RegexPattern {
    fn parse(info: Option<&str>, path: Option<&str>) -> Self {
        Self {
            path: path.expect("regex pattern needs path").into(),
            pattern: str_to_regex(info.expect("regex pattern needs pattern"))
                .expect("invalid pattern for regex pattern"),
        }
    }

    async fn load_version(&self) -> String {
        let content = tokio::fs::read_to_string(&self.path)
            .await
            .expect("reading file");
        let captures = self.pattern.captures(&content)
            .expect("not matched with the regex");
        captures.name("version").unwrap().as_str().to_string()
    }

    async fn set_version(&self, version: &str) {
        let content = tokio::fs::read_to_string(&self.path)
            .await
            .expect("reading file");
        let new_content = self.pattern.replace(&content, SetVersion(version));
        tokio::fs::write(&self.path, new_content.as_bytes())
            .await
            .expect("writing file");
    }
}

struct SetVersion<'a>(&'a str);

impl Replacer for SetVersion<'_> {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        dst.push_str(&caps["prefix"]);
        dst.push_str(self.0);
        dst.push_str(&caps["suffix"]);
    }
}
