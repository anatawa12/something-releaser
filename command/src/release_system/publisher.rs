use rand::Rng;
use tokio::fs::{read_to_string, File};
use tokio::io::AsyncWriteExt;

use crate::release_system::command_builder::{CommandBuilderMap, GradleBuilder};
use crate::release_system::VersionInfo;
use crate::*;
use std::path::PathBuf;

type StaticStrSlice = &'static [&'static str];

#[async_trait()]
pub trait Publisher {
    async fn prepare_environment(&self, version_info: &VersionInfo) -> ();
    async fn publish_project(&self, builders: &mut CommandBuilderMap) -> ();
    fn name(&self) -> &'static str;
    fn secrets(&self) -> StaticStrSlice;
}

async fn create_init_script(body: &str, prefix: &str) -> PathBuf {
    let init_d = gradle_home().join("init.d");
    tokio::fs::create_dir_all(&init_d)
        .await
        .expect("failed to create init.d");

    let rand = rand::thread_rng().gen::<[u8; 32]>();
    let rand = to_hex(&rand);

    let init_script = init_d.join(format!("{}.{}.init.gradle", prefix, rand));
    let mut file = File::create(&init_script)
        .await
        .expect("failed to open init script file");
    file.write_all(body.as_bytes())
        .await
        .expect("failed to write init script");
    drop(file);
    init_script
}

fn gradle_home() -> PathBuf {
    std::env::var_os("GRADLE_USER_HOME")
        .map(|x| PathBuf::from(x))
        .or_else(|| home::home_dir().map(|x| x.join(".gradle")))
        .expect("no GRADLE_USER_HOME found")
}

fn to_hex(data: &[u8]) -> String {
    let mut hex = Vec::with_capacity(data.len() * 2);
    for i in 0..data.len() {
        hex.push(b"0123456789abcdef"[(data[i] >> 4) as usize & 0xF]);
        hex.push(b"0123456789abcdef"[(data[i] >> 0) as usize & 0xF]);
    }
    unsafe { String::from_utf8_unchecked(hex) }
}

#[test]
fn hex() {
    assert_eq!(&to_hex(b"\x00\x01\x02\x03"), "00010203");
    assert_eq!(&to_hex(b"\x0a\x0b\x0c\x0d"), "0a0b0c0d");
    assert_eq!(&to_hex(b"\x64\x65\x66\x67"), "64656667");
    assert_eq!(&to_hex(b"\xfc\xfd\xfe\xff"), "fcfdfeff");
}

pub(super) struct GradleMavenPublisher;

#[async_trait()]
impl Publisher for GradleMavenPublisher {
    async fn prepare_environment(&self, _version_info: &VersionInfo) -> () {
        let auth = std::env::var("GRADLE_MAVEN_AUTH").expect("no GRADLE_MAVEN_AUTH env var");
        let (user, pass) = auth
            .split_once(":")
            .expect("invalid GRADLE_MAVEN_AUTH: no ':' in string");
        let pgp_key = std::env::var("GPG_PRIVATE_KEY").expect("no GPG_PRIVATE_KEY env var");
        let pgp_pass = std::env::var("GPG_PRIVATE_PASS").expect("no GPG_PRIVATE_PASS env var");

        // verify user and pass
        {
            reqwest::Client::builder()
                .build()
                .expect("failed to create http client")
                .get("https://oss.sonatype.org/service/local/status")
                .basic_auth(user, Some(pass))
                .send()
                .await
                .expect("token check http request failed")
                .error_for_status()
                .expect("invalid response! make sure tokens are valid");
        }

        let body = format!(
            include_out_str!("templates/gradle-maven.init.gradle"),
            pgp_key = pgp_key.escape_groovy(),
            pgp_pass = pgp_pass.escape_groovy(),
            user = user.escape_groovy(),
            pass = pass.escape_groovy(),
        );
        let init_script = create_init_script(&body, "maven-publish").await;
        trace!("init script created at {}", init_script.display());
    }

    async fn publish_project(&self, builders: &mut CommandBuilderMap) -> () {
        builders.find_mut::<GradleBuilder>().run_tasks(&["publish"]);
    }

    fn name(&self) -> &'static str {
        "gradle maven publisher"
    }

    fn secrets(&self) -> StaticStrSlice {
        &["GRADLE_MAVEN_AUTH", "GPG_PRIVATE_KEY", "GPG_PRIVATE_PASS"]
    }
}

pub(super) struct GradlePluginPortalPublisher;

#[async_trait()]
impl Publisher for GradlePluginPortalPublisher {
    async fn prepare_environment(&self, _version_info: &VersionInfo) -> () {
        let auth = std::env::var("GRADLE_PUBLISH_AUTH").expect("no GRADLE_PUBLISH_AUTH env var");
        let (publish_key, publish_secret) = auth
            .split_once(":")
            .expect("invalid GRADLE_PUBLISH_AUTH: no ':' in string");

        let properties_path = gradle_home().join("gradle.properties");
        let properties_file_string = match read_to_string(&properties_path).await {
            Ok(str) => str,
            // not found -> empty
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => panic!("failed to open properties file: {:?}", e),
        };
        let mut properties_file = PropertiesFile::parse(&properties_file_string)
            .expect("failed to parse properties file");
        properties_file.set_value("gradle.publish.key".to_owned(), publish_key.to_owned());
        properties_file.set_value(
            "gradle.publish.secret".to_owned(),
            publish_secret.to_owned(),
        );

        properties_file
            .write_async_file(&properties_path, properties_file_string.len() + 256)
            .await
            .expect("writing");
    }

    async fn publish_project(&self, builders: &mut CommandBuilderMap) -> () {
        builders
            .find_mut::<GradleBuilder>()
            .run_tasks(&["publishPlugins"]);
    }

    fn name(&self) -> &'static str {
        "gradle plugin portal publisher"
    }

    fn secrets(&self) -> StaticStrSlice {
        &["GRADLE_PUBLISH_AUTH"]
    }
}

pub(super) struct GradleIntellijPublisher;

#[async_trait()]
impl Publisher for GradleIntellijPublisher {
    async fn prepare_environment(&self, _version_info: &VersionInfo) -> () {
        let token =
            std::env::var("GRADLE_INTELLIJ_TOKEN").expect("no GRADLE_INTELLIJ_TOKEN env var");

        let body = format!(
            include_out_str!("templates/intellij-publish.init.gradle"),
            token = token.escape_groovy(),
        );
        let init_script = create_init_script(&body, "intellij-publish").await;
        trace!("init script created at {}", init_script.display());
    }

    async fn publish_project(&self, builders: &mut CommandBuilderMap) -> () {
        builders
            .find_mut::<GradleBuilder>()
            .run_tasks(&["publishPlugin"]);
    }

    fn name(&self) -> &'static str {
        "gradle maven publisher"
    }

    fn secrets(&self) -> StaticStrSlice {
        &["GRADLE_INTELLIJ_TOKEN"]
    }
}

types_enum!(Publisher {
    GradleMavenPublisher: "gradle-maven",
    GradlePluginPortalPublisher: "gradle-plugin-portal",
    GradleIntellijPublisher: "gradle-intellij-publisher",
});
