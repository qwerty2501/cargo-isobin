use std::path::PathBuf;

use tokio::fs;

use crate::paths::isobin_config::IsobinConfigPathError;
use crate::paths::project::Project;
use crate::paths::workspace::WorkspaceProvider;
use crate::{Result, ServiceOption};

pub struct PathService {
    workspace_provider: WorkspaceProvider,
}

impl Default for PathService {
    fn default() -> Self {
        Self {
            workspace_provider: WorkspaceProvider::new(Project::default()),
        }
    }
}

impl PathService {
    pub async fn path(&self, service_option: ServiceOption) -> Result<PathBuf> {
        let isobin_config_dir = service_option
            .isobin_config_path()
            .parent()
            .ok_or(IsobinConfigPathError::NotFoundIsobinConfig)?;
        let isobin_config_dir = fs::canonicalize(isobin_config_dir).await?;
        let workspace = self
            .workspace_provider
            .base_unique_workspace_dir_from_isobin_config_dir(&isobin_config_dir)
            .await?;
        Ok(workspace.bin_dir().into())
    }
}
