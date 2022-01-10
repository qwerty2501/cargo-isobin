use crate::paths::workspace::Workspace;
use async_std::path::PathBuf;
use nanoid::nanoid;

#[derive(Getters)]
pub struct VirtualHome {
    #[allow(dead_code)]
    base_dir: PathBuf,
}

impl VirtualHome {
    #[allow(dead_code)]
    pub fn from_workspace_tmp(workspace: &Workspace) -> Self {
        Self {
            base_dir: workspace.tmp_dir().join(nanoid!()),
        }
    }
    #[allow(dead_code)]
    pub fn from_workspace_home(workspace: &Workspace) -> Self {
        Self {
            base_dir: workspace.home_dir().clone(),
        }
    }
}
