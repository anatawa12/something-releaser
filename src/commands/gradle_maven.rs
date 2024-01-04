use crate::utils::gradle::{escape_groovy_string, gradle_home_dir};
use crate::utils::write_to_new_file;
use crate::CmdResult;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "gradle-maven")]
#[command(no_binary_name = true)]
/// Configure gradle globally to publish to maven repository
pub(crate) struct GradleMaven {
    #[arg(long = "url")]
    pub url: Option<String>,
    #[arg(long)]
    pub user: String,
    #[arg(long)]
    pub pass: String,
    #[arg(value_name = "URL")]
    pub url_positional: Option<String>,
}

impl GradleMaven {
    fn generate_init_script(&self, url: &str) -> String {
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
            escape_groovy_string(url)
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

    pub async fn configure(self) -> CmdResult {
        let url = self
            .url
            .as_ref()
            .or(self.url_positional.as_ref())
            .expect("no url specified");

        let mut path = gradle_home_dir();
        path.push("init.d");
        path.push(format!("gradle-maven.{}.gradle", uuid::Uuid::new_v4()));
        let path = path;

        write_to_new_file(path, self.generate_init_script(url).as_bytes())
            .await
            .expect("failed to create init script");

        ok!()
    }
}

#[test]
fn generated_init_script() {
    let maven = GradleMaven {
        url: None,
        url_positional: None,
        user: "sonatype-test".into(),
        pass: "sonatype-password".into(),
    };

    assert_eq!(
        maven.generate_init_script("https://oss.sonatype.org/service/local/staging/deploy/maven2/"),
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

    put_server![
        server (auth: "Basic c29uYXR5cGUtdGVzdDpzb25hdHlwZS1wYXNzd29yZA==") =>
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.sha1",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.md5",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.sha256",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.sha512",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.sha1",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.md5",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.sha256",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.sha512",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.sha1",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.md5",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.sha256",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.sha512",
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml",
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml.sha1",
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml.md5",
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml.sha256",
        "/com/anatawa12/something-releaser/test/publish/maven-metadata.xml.sha512",
    ];

    server.expect(
        Expectation::matching(all_of![request::method("GET")])
            .times(..)
            .respond_with(status_code(404)),
    );

    let new_home = tempfile::tempdir().unwrap();
    std::env::set_var("GRADLE_USER_HOME", new_home.path());

    // execute our code
    let maven = GradleMaven {
        url: server.url("/").to_string().into(),
        user: "sonatype-test".into(),
        pass: "sonatype-password".into(),
        url_positional: None,
    };

    maven.configure().await.unwrap();

    // test run
    let result = tokio::process::Command::new("./gradlew")
        .args(["--no-daemon", "publish"])
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
