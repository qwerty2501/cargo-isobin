use tokio::fs;

use super::*;
use crate::paths::isobin_config::search_isobin_config_path;
use std::path::PathBuf;

#[derive(Getters)]
pub(crate) struct ServiceOption {
    isobin_config_path: PathBuf,
}

#[derive(Default)]
pub(crate) struct ServiceOptionBuilder {
    isobin_config_path: Option<PathBuf>,
}

impl ServiceOptionBuilder {
    pub fn isobin_config_path(self, isobin_config_path: Option<PathBuf>) -> Self {
        Self { isobin_config_path }
    }

    pub async fn try_build(self) -> Result<ServiceOption> {
        let isobin_config_path = if let Some(isobin_config_path) = self.isobin_config_path {
            isobin_config_path
        } else {
            let current_dir = std::env::current_dir().unwrap();
            search_isobin_config_path(current_dir).await?
        };
        let isobin_config_path = fs::canonicalize(isobin_config_path).await?;
        Ok(ServiceOption { isobin_config_path })
    }
}
