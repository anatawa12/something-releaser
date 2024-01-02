use crate::utils::gradle::{escape_groovy_string, gradle_home_dir};
use tokio::io::AsyncWriteExt;

pub(crate) struct GradleSigning {
    pub key: String,
    pub pass: String,
}

impl GradleSigning {
    fn generate_init_script(&self) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        writeln!(s, "afterProject {{ proj ->").unwrap();
        writeln!(
            s,
            "  if (proj.plugins.findPlugin(\"org.gradle.publishing\") == null) return"
        )
        .unwrap();
        writeln!(s, "  proj.apply {{").unwrap();
        writeln!(s, "    plugin(\"signing\")").unwrap();
        writeln!(s, "  }}").unwrap();
        writeln!(
            s,
            "  proj.signing.useInMemoryPgpKeys(\"{}\", \"{}\")",
            escape_groovy_string(&self.key),
            escape_groovy_string(&self.pass),
        )
        .unwrap();
        writeln!(
            s,
            "  proj.publishing.publications.forEach {{ publication ->"
        )
        .unwrap();
        writeln!(s, "    proj.signing.sign(publication)").unwrap();
        writeln!(s, "  }}").unwrap();
        writeln!(s, "}}").unwrap();

        s
    }

    pub async fn configure(&self) {
        let mut path = gradle_home_dir();
        path.push("init.d");
        path.push("gradle-signing.gradle");
        let path = path;

        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("failed to create init.d directory");
        let mut file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await
            .expect("failed to create init script (might be already configured)");
        file.write_all(self.generate_init_script().as_bytes())
            .await
            .expect("failed to write init script");
        file.flush().await.expect("failed to flush init script");
    }
}

#[tokio::test]
async fn generated_init_script() {
    let key = key_file().await;
    let signing = GradleSigning {
        key: key.clone(),
        pass: "".to_owned(),
    };

    assert_eq!(
        signing.generate_init_script(),
        format!(
            r##"afterProject {{ proj ->
  if (proj.plugins.findPlugin("org.gradle.publishing") == null) return
  proj.apply {{
    plugin("signing")
  }}
  proj.signing.useInMemoryPgpKeys("{key}", "")
  proj.publishing.publications.forEach {{ publication ->
    proj.signing.sign(publication)
  }}
}}
"##,
            key = key.replace("'", "\\'").replace("\n", "\\n")
        )
    )
}

#[tokio::test]
async fn test_with_project() {
    use crate::commands::gradle_maven::GradleMaven;
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
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.asc",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.asc.sha1",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.asc.md5",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.asc.sha256",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.jar.asc.sha512",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.sha1",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.md5",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.sha256",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.sha512",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.asc",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.asc.sha1",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.asc.md5",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.asc.sha256",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.pom.asc.sha512",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.sha1",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.md5",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.sha256",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.sha512",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.asc",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.asc.sha1",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.asc.md5",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.asc.sha256",
        "/com/anatawa12/something-releaser/test/publish/unspecified/publish-unspecified.module.asc.sha512",
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
        url: server.url("/").to_string(),
        user: "sonatype-test".into(),
        pass: "sonatype-password".into(),
    };
    let sign = GradleSigning {
        key: key_file().await,
        pass: "".to_owned(),
    };

    maven.configure().await;
    sign.configure().await;

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

#[cfg(test)]
async fn key_file() -> String {
    tokio::fs::read_to_string("__tests__resources/gpg/bob.secret-key.asc")
        .await
        .unwrap()
}
