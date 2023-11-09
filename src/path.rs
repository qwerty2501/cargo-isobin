use std::path::PathBuf;

use crate::{
    paths::{isobin_config::isobin_config_dir, workspace::WorkspaceProvider},
    service_option::ServiceOptionBuilder,
    Result,
};

#[derive(Default)]
pub struct PathService {
    workspace_provider: WorkspaceProvider,
}

impl PathService {
    pub async fn path(&self, path_service_option: PathServiceOption) -> Result<PathBuf> {
        let isobin_config_dir = isobin_config_dir(path_service_option.isobin_config_path())?;
        let workspace = self
            .workspace_provider
            .base_unique_workspace_dir_from_isobin_config_dir(isobin_config_dir)
            .await?;
        Ok(workspace.bin_dir().into())
    }
}

#[derive(Getters)]
pub struct PathServiceOption {
    isobin_config_path: PathBuf,
}

#[derive(Default)]
pub struct PathServiceOptionBuilder {
    service_option_builder: ServiceOptionBuilder,
}

impl PathServiceOptionBuilder {
    pub fn isobin_config_path(mut self, isobin_config_path: Option<PathBuf>) -> Self {
        self.service_option_builder = self
            .service_option_builder
            .isobin_config_path(isobin_config_path);
        self
    }
    pub async fn try_build(self) -> Result<PathServiceOption> {
        let service_option = self.service_option_builder.try_build().await?;
        Ok(PathServiceOption {
            isobin_config_path: service_option.isobin_config_path().clone(),
        })
    }
}
