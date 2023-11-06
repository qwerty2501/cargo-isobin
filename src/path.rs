use std::path::PathBuf;

use crate::{
    paths::{isobin_config::isobin_config_dir, workspace::WorkspaceProvider},
    Result, ServiceOption,
};

#[derive(Default)]
pub struct PathService {
    workspace_provider: WorkspaceProvider,
}

impl PathService {
    pub async fn path(&self, service_option: ServiceOption) -> Result<PathBuf> {
        let isobin_config_dir = isobin_config_dir(service_option.isobin_config_path())?;
        let workspace = self
            .workspace_provider
            .base_unique_workspace_dir_from_isobin_config_dir(isobin_config_dir)
            .await?;
        Ok(workspace.bin_dir().into())
    }
}
