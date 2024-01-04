use crate::utils::gradle::gradle_home_dir;
use crate::utils::properties::PropertiesFile;
use crate::utils::write_to_new_file;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(no_binary_name = true)]
#[clap(name = "prepare-gradle-plugin-portal")]
pub(crate) struct GradlePluginPortal {
    pub key: String,
    pub secret: String,
}

impl GradlePluginPortal {
    fn set_properties(&self, properties: &mut PropertiesFile) {
        properties.set("gradle.publish.key", self.key.to_owned());
        properties.set("gradle.publish.secret", self.secret.to_owned());
    }

    fn generate_init_script(&self) -> &'static str {
        concat!(
            "afterProject {{ proj ->\n",
            "  if (proj.plugins.findPlugin(\"com.gradle.plugin-publish\") == null) return\n",
            "  if (proj.pluginBundle.metaClass.hasProperty(\"mavenCoordinates\")) {{\n",
            "    if (proj.pluginBundle.mavenCoordinates.groupId == null) {{\n",
            "      throw new Exception(\"mavenCoordinates.groupId is not specified!\")\n",
            "    }}\n",
            "  }}\n",
            "}}\n",
        )
    }

    pub async fn configure(&self) {
        let gradle_home = gradle_home_dir();

        // properties
        let properties_path = gradle_home.join("gradle.properties");
        let mut properties = PropertiesFile::load_may_not_exist(&properties_path)
            .await
            .expect("reading gradle.properties");
        self.set_properties(&mut properties);
        tokio::fs::write(properties_path, properties.to_string().as_bytes())
            .await
            .expect("writing gradle.properties");

        // init
        write_to_new_file(
            gradle_home.join("init.d/gradle-plugin-portal.gradle"),
            self.generate_init_script().as_bytes(),
        )
        .await
        .expect("failed to create init script");
    }
}

#[test]
fn properties_file() {
    let mut properties = PropertiesFile::new();
    let portal = GradlePluginPortal {
        key: "gradle-portal-key".into(),
        secret: "gradle-portal-secret".into(),
    };
    portal.set_properties(&mut properties);

    assert_eq!(
        properties.get("gradle.publish.key"),
        Some("gradle-portal-key".into())
    );
    assert_eq!(
        properties.get("gradle.publish.secret"),
        Some("gradle-portal-secret".into())
    );
}

#[tokio::test]
async fn test_with_file_system() {
    let new_home = tempfile::tempdir().unwrap();
    std::env::set_var("GRADLE_USER_HOME", new_home.path());

    let portal = GradlePluginPortal {
        key: "gradle-portal-key".into(),
        secret: "gradle-portal-secret".into(),
    };

    portal.configure().await;

    let properties_file =
        std::fs::read_to_string(new_home.path().join("gradle.properties")).unwrap();
    assert!(properties_file.contains("gradle.publish.key=gradle-portal-key"));
    assert!(properties_file.contains("gradle.publish.secret=gradle-portal-secret"));
}
