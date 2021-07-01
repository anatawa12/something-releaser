use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::*;

use super::CommandBuilder;

#[derive(Default)]
pub struct GradleBuilder {
    props: HashMap<String, String>,
    tasks: Vec<String>,
    init_scripts: Vec<PathBuf>,
}

impl GradleBuilder {
    pub fn add_property(
        &mut self,
        name: impl IntoString,
        value: impl IntoString,
    ) -> &mut GradleBuilder {
        self.props.insert(name.into_string(), value.into_string());
        self
    }

    pub fn run_tasks(
        &mut self,
        names: impl IntoIterator<Item = impl IntoString>,
    ) -> &mut GradleBuilder {
        for name in names {
            self.tasks.push(name.into_string());
        }
        self
    }

    pub fn add_init_script(&mut self, name: impl Into<PathBuf>) -> &mut GradleBuilder {
        self.init_scripts.push(name.into());
        self
    }
}

impl CommandBuilder for GradleBuilder {
    fn create_command_to_exec(&self, dry_run: bool) -> Command {
        let mut command = Command::new("./gradlew");
        if dry_run {
            command.arg("--dry-run");
        }
        for (key, value) in &self.props {
            command.arg(format!("-P{}={}", key, value));
        }
        for init_script in &self.init_scripts {
            command.arg(format!("-I{}", init_script.display()));
        }
        command.arg("--");
        for task in &self.tasks {
            command.arg(task);
        }
        command
    }

    fn name(&self) -> &'static str {
        "gradle"
    }
}
