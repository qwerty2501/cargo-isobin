use crate::paths::workspace::Workspace;
use std::path::PathBuf;

use super::*;

#[derive(Getters, Clone)]
pub struct CargoWorkspace {
    #[allow(dead_code)]
    cargo_home_dir: PathBuf,
}

impl CargoWorkspace {
    pub fn from_workspace(workspace: &Workspace) -> Self {
        let cargo_home_dir = workspace.home_dir().join(PROVIDER_NAME);
        Self { cargo_home_dir }
    }
}
