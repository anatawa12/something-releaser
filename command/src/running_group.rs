use std::fmt::Display;

pub(crate) struct RunningGroup(());

pub(crate) fn start_group(name: impl Display) -> RunningGroup {
    if crate::is_actions_env() {
        println!("::group::{}", name);
    } else {
        crate::info!("start {}", name);
    }
    RunningGroup(())
}

impl Drop for RunningGroup {
    fn drop(&mut self) {
        if crate::is_actions_env() {
            println!("::endgroup::");
        }
    }
}
