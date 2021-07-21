use std::process::Command;

use crate::*;

use super::CommandBuilder;

#[derive(Default)]
pub struct GradleBuilder {
    tasks: Vec<String>,
}

impl GradleBuilder {
    pub fn run_tasks(
        &mut self,
        names: impl IntoIterator<Item = impl IntoString>,
    ) -> &mut GradleBuilder {
        for name in names {
            self.tasks.push(name.into_string());
        }
        self
    }
}

impl CommandBuilder for GradleBuilder {
    fn create_command_to_exec(&self, dry_run: bool) -> Command {
        let mut command = Command::new("./gradlew");
        if dry_run {
            command.arg("--dry-run");
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
