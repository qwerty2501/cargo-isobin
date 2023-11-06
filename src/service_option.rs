use tokio::fs;

use super::*;
use crate::paths::isobin_config::{search_isobin_config_path, IsobinConfigPathError};
use crate::paths::workspace::{Workspace, WorkspaceProvider};
use crate::IsobinConfig;
use std::path::{Path, PathBuf};

#[derive(Getters)]
pub struct ServiceOption {
    isobin_config_path: PathBuf,
    isobin_config_dir: PathBuf,
    isobin_config: IsobinConfig,
    workspace: Workspace,
}

#[derive(Default)]
pub struct ServiceOptionBuilder {
    isobin_config_path: Option<PathBuf>,
    workspace_provider: WorkspaceProvider,
}

impl ServiceOptionBuilder {
    pub fn isobin_config_path(self, isobin_config_path: impl AsRef<Path>) -> Self {
        Self {
            isobin_config_path: Some(isobin_config_path.as_ref().into()),
            workspace_provider: WorkspaceProvider::default(),
        }
    }

    pub async fn try_build(self) -> Result<ServiceOption> {
        let isobin_config_path = if let Some(isobin_config_path) = self.isobin_config_path {
            isobin_config_path
        } else {
            let current_dir = std::env::current_dir().unwrap();
            search_isobin_config_path(current_dir).await?
        };
        let isobin_config_path = fs::canonicalize(isobin_config_path).await?;
        let mut isobin_config = IsobinConfig::parse_from_file(&isobin_config_path).await?;
        let isobin_config_dir = isobin_config_path
            .parent()
            .ok_or(IsobinConfigPathError::NotFoundIsobinConfig)?;
        isobin_config.fix(isobin_config_dir);
        isobin_config.validate()?;
        let workspace = self
            .workspace_provider
            .base_unique_workspace_dir_from_isobin_config_dir(isobin_config_dir)
            .await?;
        Ok(ServiceOption {
            isobin_config_path: isobin_config_path.clone(),
            isobin_config,
            isobin_config_dir: isobin_config_dir.into(),
            workspace,
        })
    }
}
