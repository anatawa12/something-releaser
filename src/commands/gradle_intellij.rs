use crate::utils::gradle::escape_groovy_string;
use crate::CmdResult;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(no_binary_name = true)]
#[command(name = "prepare-gradle-intellij")]
/// Configure gradle globally to publish to intellij plugin portal
pub(crate) struct GradleIntellij {
    token: String,
}

impl GradleIntellij {
    fn generate_init_script(&self) -> String {
        format!(
            concat!(
                "afterProject {{ proj ->\n",
                "  if (proj.plugins.findPlugin(\"org.jetbrains.intellij\") == null) return\n",
                "  proj.tasks.publishPlugin {{\n",
                "    token = \"{token}\"\n",
                "  }}\n",
                "}}\n",
            ),
            token = escape_groovy_string(&self.token)
        )
    }

    pub async fn configure(&self) -> CmdResult {
        let mut path = crate::utils::gradle::gradle_home_dir();
        path.push("init.d/gradle-intellij.gradle");
        let path = path;

        crate::utils::write_to_new_file(path, self.generate_init_script().as_bytes())
            .await
            .expect("failed to create init script");

        ok!()
    }
}

#[test]
fn generated_init_script() {
    let intellij = GradleIntellij {
        token: "gradle-intellij-token-here".into(),
    };

    assert_eq!(
        intellij.generate_init_script(),
        concat!(
            "afterProject { proj ->\n",
            "  if (proj.plugins.findPlugin(\"org.jetbrains.intellij\") == null) return\n",
            "  proj.tasks.publishPlugin {\n",
            "    token = \"gradle-intellij-token-here\"\n",
            "  }\n",
            "}\n",
        )
    );
}
