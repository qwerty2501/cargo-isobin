use crate::paths::workspace::Workspace;
use std::path::PathBuf;

#[derive(Getters, Clone)]
pub struct CargoWorkspace {
    #[allow(dead_code)]
    cargo_home_dir: PathBuf,
    cargo_bin_dir: PathBuf,
}

impl CargoWorkspace {
    pub fn from_workspace(workspace: &Workspace) -> Self {
        let cargo_home_dir = workspace.home_dir().join("cargo");
        let cargo_bin_dir = cargo_home_dir.join("bin");
        Self {
            cargo_bin_dir,
            cargo_home_dir,
        }
    }
}
