use std::path::PathBuf;

use crate::{
    paths::{
        isobin_config::{isobin_config_dir, isobin_config_path_canonicalize},
        workspace::WorkspaceProvider,
    },
    Result,
};

#[derive(Default)]
pub struct PathService {
    workspace_provider: WorkspaceProvider,
}

impl PathService {
    pub async fn path(&self, path_service_option: PathServiceOption) -> Result<PathBuf> {
        let path_service_option = path_service_option.fix().await?;
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
    isobin_config_path: Option<PathBuf>,
}

impl PathServiceOption {
    pub async fn fix(self) -> Result<FixedPathServiceOption> {
        let isobin_config_path = isobin_config_path_canonicalize(self.isobin_config_path).await?;
        Ok(FixedPathServiceOption { isobin_config_path })
    }
}

#[derive(Getters)]
pub struct FixedPathServiceOption {
    isobin_config_path: PathBuf,
}

#[derive(Default)]
pub struct PathServiceOptionBuilder {
    isobin_config_path: Option<PathBuf>,
}

impl PathServiceOptionBuilder {
    pub fn isobin_config_path(mut self, isobin_config_path: PathBuf) -> Self {
        self.isobin_config_path = Some(isobin_config_path);
        self
    }
    pub fn build(self) -> PathServiceOption {
        PathServiceOption {
            isobin_config_path: self.isobin_config_path,
        }
    }
}
