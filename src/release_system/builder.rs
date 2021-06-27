use std::path::Path;

use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::release_system::VersionInfo;
use crate::*;

#[async_trait()]
pub trait Builder {
    async fn build_project(&self, project: &Path, version_info: &VersionInfo) -> ();
    fn name(&self) -> &'static str;
}

pub(super) struct GradleBuilder;

#[async_trait()]
impl Builder for GradleBuilder {
    async fn build_project(&self, project: &Path, version_info: &VersionInfo) -> () {
        let home = home::home_dir().expect("no home directory found.");
        let init_d = home.join(".gradle").join("init.d");
        tokio::fs::create_dir_all(&init_d)
            .await
            .expect("creating init.d");
        let init_script = init_d.join("release_note_properties.gradle");
        let init_script = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&init_script)
            .await
            .expect("creating release note init script");
        let mut init_script = BufWriter::new(init_script);
        let buf = format!(
            r#"
beforeProject {{
    ext.set("com.anatawa12.releaser.release-note.html", '{}')
    ext.set("com.anatawa12.releaser.release-note.markdown", '{}')
}}
"#,
            escape(&version_info.release_note_html),
            escape(&version_info.release_note_markdown)
        );
        init_script
            .write_all(buf.as_bytes())
            .await
            .expect("writing release note init script");
        init_script
            .flush()
            .await
            .expect("writing release note init script");
        drop(init_script);

        GradleWrapperHelper::new(project)
            .run_tasks(&["build"])
            .await
            .expect("./gradlew build");
    }

    fn name(&self) -> &'static str {
        "gradle with wrapper"
    }
}

fn escape(str: &str) -> String {
    let mut builder = Vec::<u8>::with_capacity(str.len());
    for x in str.bytes() {
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

types_enum!(Builder { GradleBuilder });
