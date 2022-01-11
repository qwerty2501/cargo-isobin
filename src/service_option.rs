use crate::paths::isobin_config::search_isobin_config_path;
use crate::paths::isobin_config::IsobinConfigPathError;
use crate::{IsobinConfig, IsobinConfigError};
use async_std::path::{Path, PathBuf};

#[derive(Getters)]
pub struct ServiceOption {
    isobin_config: IsobinConfig,
}

pub struct ServiceOptionBuilder {
    isobin_config_path: Option<PathBuf>,
}

impl ServiceOptionBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            isobin_config_path: None,
        }
    }
    pub fn isobin_config_path(self, isobin_config_path: impl AsRef<Path>) -> Self {
        Self {
            isobin_config_path: Some(isobin_config_path.as_ref().into()),
        }
    }

    pub async fn try_build(self) -> Result<ServiceOption> {
        let isobin_config_path = if let Some(isobin_config_path) = self.isobin_config_path {
            isobin_config_path
        } else {
            let current_dir = std::env::current_dir().unwrap();
            search_isobin_config_path(current_dir).await?
        };
        let isobin_config = IsobinConfig::parse_from_file(isobin_config_path).await?;
        Ok(ServiceOption { isobin_config })
    }
}

type Result<T> = std::result::Result<T, ServiceOptionBuildError>;

#[derive(thiserror::Error, Debug, new)]
pub enum ServiceOptionBuildError {
    #[error("{0}")]
    IsobinPath(#[from] IsobinConfigPathError),
    #[error("{0}")]
    IsobinConfig(#[from] IsobinConfigError),
}
