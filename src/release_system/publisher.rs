use std::path::Path;

use rand::Rng;
use tempfile::NamedTempFile;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::release_system::VersionInfo;
use crate::*;

#[async_trait()]
pub trait Publisher {
    async fn publish_project(
        &self,
        project: &Path,
        version_info: &VersionInfo,
        dry_run: bool,
    ) -> ();
    fn name(&self) -> &'static str;
}

async fn create_temp_init_script(body: &str, prefix: &str) -> NamedTempFile {
    let init_script = tempfile::Builder::new()
        .prefix(prefix)
        .suffix(".init.gradle")
        .tempfile()
        .expect("failed to create a temporal init script file.");
    let mut file = File::create(init_script.path())
        .await
        .expect("failed to open init script file");
    file.write_all(body.as_bytes())
        .await
        .expect("failed to write init script");
    drop(file);
    return init_script;
}

pub(super) struct GradleMavenPublisher;

#[async_trait()]
impl Publisher for GradleMavenPublisher {
    async fn publish_project(
        &self,
        project: &Path,
        _version_info: &VersionInfo,
        dry_run: bool,
    ) -> () {
        let auth = std::env::var("GRADLE_MAVEN_AUTH").expect("no GRADLE_MAVEN_AUTH env var");
        let (user, pass) = auth
            .split_once(":")
            .expect("invalid GRADLE_MAVEN_AUTH: no ':' in string");
        let pgp_key = std::env::var("GPG_PRIVATE_KEY").expect("no GPG_PRIVATE_KEY env var");
        let pgp_pass = std::env::var("GPG_PRIVATE_PASS").expect("no GPG_PRIVATE_PASS env var");
        let random = rand::thread_rng().gen_ascii_rand(20);
        let (rand0, rand1) = random.split_at(rand::thread_rng().gen_range(0..random.len()));

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

        if dry_run {
            warn!("dry run! no publish task invocation");
            return;
        }

        let body = format!(
            include_out_str!("templates/gradle-maven.init.gradle"),
            rand0 = rand0,
            rand1 = rand1,
        );
        let init_script = create_temp_init_script(&body, "maven-publish").await;
        trace!("init script created at {}", init_script.path().display());
        GradleWrapperHelper::new(project)
            .add_init_script(init_script.path())
            .add_property(format!("{}pgp_key{}", rand0, rand1), pgp_key)
            .add_property(format!("{}pgp_pass{}", rand0, rand1), pgp_pass)
            .add_property(format!("{}user{}", rand0, rand1), user)
            .add_property(format!("{}pass{}", rand0, rand1), pass)
            .run_tasks(&["publish"])
            .await
            .expect("./gradlew publish");
    }

    fn name(&self) -> &'static str {
        "gradle maven publisher"
    }
}

pub(super) struct GradlePluginPortalPublisher;

#[async_trait()]
impl Publisher for GradlePluginPortalPublisher {
    async fn publish_project(
        &self,
        project: &Path,
        _version_info: &VersionInfo,
        dry_run: bool,
    ) -> () {
        let auth = std::env::var("GRADLE_PUBLISH_AUTH").expect("no GRADLE_PUBLISH_AUTH env var");
        let (publish_key, publish_secret) = auth
            .split_once(":")
            .expect("invalid GRADLE_PUBLISH_AUTH: no ':' in string");

        if dry_run {
            warn!("dry run! no publishPlugins task invocation");
            return;
        }

        GradleWrapperHelper::new(project)
            .add_property("gradle.publish.key", publish_key)
            .add_property("gradle.publish.secret", publish_secret)
            .run_tasks(&["publishPlugins"])
            .await
            .expect("./gradlew publishPlugins");
    }

    fn name(&self) -> &'static str {
        "gradle plugin portal publisher"
    }
}

pub(super) struct GradleIntellijPublisher;

#[async_trait()]
impl Publisher for GradleIntellijPublisher {
    async fn publish_project(
        &self,
        project: &Path,
        _version_info: &VersionInfo,
        dry_run: bool,
    ) -> () {
        let token =
            std::env::var("GRADLE_INTELLIJ_TOKEN").expect("no GRADLE_INTELLIJ_TOKEN env var");
        let random = rand::thread_rng().gen_ascii_rand(20);
        let (rand0, rand1) = random.split_at(rand::thread_rng().gen_range(0..random.len()));

        if dry_run {
            warn!("dry run! no publishPlugin task invocation");
            return;
        }

        let body = format!(
            include_out_str!("templates/intellij-publish.init.gradle"),
            rand0 = rand0,
            rand1 = rand1,
        );
        let init_script = create_temp_init_script(&body, "intellij-publish").await;
        trace!("init script created at {}", init_script.path().display());

        GradleWrapperHelper::new(project)
            .add_init_script(init_script.path())
            .add_property(format!("{}token{}", rand0, rand1), token)
            .run_tasks(&["publishPlugin"])
            .await
            .expect("./gradlew publishPlugin");
    }

    fn name(&self) -> &'static str {
        "gradle maven publisher"
    }
}

types_enum!(Publisher {
    GradleMavenPublisher,
    GradlePluginPortalPublisher,
    GradleIntellijPublisher,
});
