use crate::utils::gradle::{escape_groovy_string, gradle_home_dir};
use tokio::io::AsyncWriteExt;

pub(crate) struct GradleMaven {
    pub url: String,
    pub user: String,
    pub pass: String,
}

impl GradleMaven {
    fn generate_init_script(&self) -> String {
        use std::fmt::Write;
        let mut init_script = String::new();
        writeln!(init_script, "afterProject {{ proj ->").unwrap();
        writeln!(
            init_script,
            "  if (proj.plugins.findPlugin(\"org.gradle.maven-publish\") == null) return"
        )
        .unwrap();
        writeln!(init_script, "  proj.publishing.repositories.maven {{").unwrap();
        writeln!(
            init_script,
            "    url = uri(\"{}\")",
            escape_groovy_string(&self.url)
        )
        .unwrap();
        writeln!(init_script, "    // gradle may disallow insecure protocol").unwrap();
        writeln!(init_script, "    allowInsecureProtocol = true").unwrap();
        if !self.user.is_empty() {
            writeln!(init_script, "    credentials.username = \"{}\"", self.user).unwrap();
        }
        if !self.pass.is_empty() {
            writeln!(init_script, "    credentials.password = \"{}\"", self.pass).unwrap();
        }
        writeln!(init_script, "  }}").unwrap();
        writeln!(init_script, "}}").unwrap();

        init_script
    }

    pub async fn configure(&self) {
        let mut path = gradle_home_dir();
        path.push("init.d");
        path.push(format!("gradle-maven.{}.gradle", uuid::Uuid::new_v4()));
        let path = path;

        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("failed to create init.d directory");
        let mut file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await
            .expect("failed to create init script");
        file.write_all(self.generate_init_script().as_bytes())
            .await
            .expect("failed to write init script");
        file.flush().await.expect("failed to flush init script");
    }
}

#[test]
fn generated_init_script() {
    let maven = GradleMaven {
        url: "https://oss.sonatype.org/service/local/staging/deploy/maven2/".into(),
        user: "sonatype-test".into(),
        pass: "sonatype-password".into(),
    };

    assert_eq!(
        maven.generate_init_script(),
        r##"afterProject { proj ->
  if (proj.plugins.findPlugin("org.gradle.maven-publish") == null) return
  proj.publishing.repositories.maven {
    url = uri("https://oss.sonatype.org/service/local/staging/deploy/maven2/")
    // gradle may disallow insecure protocol
    allowInsecureProtocol = true
    credentials.username = "sonatype-test"
    credentials.password = "sonatype-password"
  }
}
"##
    )
}

#[tokio::test]
async fn test_with_project() {
    use httptest::matchers::*;
    use httptest::responders::*;
    use httptest::*;

    let _ = pretty_env_logger::try_init();

    // prepare
    let server = Server::run();
    fn expectation(path: &'static str) -> Expectation {
        Expectation::matching(all_of![
            request::headers(contains((
                "authorization",
                "Basic c29uYXR5cGUtdGVzdDpzb25hdHlwZS1wYXNzd29yZA=="
            ))),
            request::method_path("PUT", path),
        ])
        .respond_with(status_code(201))
    }
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.sha1",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.md5",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.sha256",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.sha512",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.sha1",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.md5",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.sha256",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.sha512",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module",
    ));
    server.expect(expectation("/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.sha1"));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.md5",
    ));
    server.expect(expectation("/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.sha256"));
    server.expect(expectation("/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.sha512"));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml.sha1",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml.md5",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml.sha256",
    ));
    server.expect(expectation(
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml.sha512",
    ));

    server.expect(
        Expectation::matching(all_of![request::method("GET")])
            .times(..)
            .respond_with(status_code(404)),
    );

    let new_home = tempfile::tempdir().unwrap();
    std::env::set_var("GRADLE_USER_HOME", new_home.path());

    // execute our code
    let maven = GradleMaven {
        url: server.url("/").to_string(),
        user: "sonatype-test".into(),
        pass: "sonatype-password".into(),
    };

    maven.configure().await;

    // test run
    let result = tokio::process::Command::new("./gradlew")
        .args(&["--no-daemon", "publish"])
        .current_dir("__tests__resources/publish-environment/gradle-maven.test.project")
        .envs(std::env::vars())
        .stdin(std::process::Stdio::null())
        .status()
        .await
        .expect("failed to run gradlew");

    if !result.success() {
        panic!("gradle exited with success");
    }
}
