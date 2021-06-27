use std::path::Path;

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
            r#"
afterProject {{ proj ->
    if (proj.plugins.findPlugin("org.gradle.maven-publish") == null) return;

    proj.apply {{
        plugin("signing")
    }}

    proj.signing {{
        useInMemoryPgpKeys("{pgp_key}", "{pgp_pass}")
        proj.publishing.publications.forEach {{ publication ->
            sign(publication)
        }}
    }}

    proj.publishing.repositories {{
        maven {{
            name = "mavenCentral"
            url = uri("https://oss.sonatype.org/service/local/staging/deploy/maven2/")

            credentials {{
                username = "{user}"
                password = "{pass}"
            }}
        }}
    }}
}}
"#,
            pgp_key = pgp_key.escape_groovy(),
            pgp_pass = pgp_pass.escape_groovy(),
            user = user.escape_groovy(),
            pass = pass.escape_groovy(),
        );
        let init_script = tempfile::Builder::new()
            .prefix("maven-publish")
            .suffix(".init.gradle")
            .tempfile()
            .expect("failed to create a init script file.");
        let mut file = File::create(init_script.path())
            .await
            .expect("failed to open init script file");
        file.write_all(body.as_bytes())
            .await
            .expect("failed to write init script");
        drop(file);
        trace!("init script created at {}", init_script.path().display());
        GradleWrapperHelper::new(project)
            .add_init_script(init_script.path())
            .run_tasks(&["publish"])
            .await
            .expect("./gradlew publish");
    }

    fn name(&self) -> &'static str {
        "gradle maven publisher"
    }
}

types_enum!(Publisher {
    GradleMavenPublisher,
});
