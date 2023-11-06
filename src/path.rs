use std::path::PathBuf;

use crate::{Result, ServiceOption};

#[derive(Default)]
pub struct PathService {}

impl PathService {
    pub async fn path(&self, service_option: ServiceOption) -> Result<PathBuf> {
        let workspace = service_option.workspace();
        Ok(workspace.bin_dir().into())
    }
}
