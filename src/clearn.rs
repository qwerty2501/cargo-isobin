use std::path::PathBuf;

use tokio::fs;

use crate::{
    paths::{
        isobin_manifest::{isobin_manifest_dir, isobin_manifest_path_canonicalize},
        workspace::WorkspaceProvider,
    },
    Result,
};

#[derive(Default)]
pub struct CleanService {
    workspace_provider: WorkspaceProvider,
}

impl CleanService {
    pub async fn clean(&self, clearn_service_option: CleanServiceOption) -> Result<()> {
        let clean_service_option = clearn_service_option.fix().await?;

        let isobin_manifest_dir = isobin_manifest_dir(clean_service_option.isobin_manifest_path())?;

        let workspace = self
            .workspace_provider
            .base_unique_workspace_dir_from_isobin_manifest_dir(isobin_manifest_dir)
            .await?;
        if workspace.base_dir().exists() && workspace.base_dir().is_dir() {
            fs::remove_dir_all(workspace.base_dir()).await?;
        }
        if workspace.cache_dir().exists() && workspace.cache_dir().is_dir() {
            fs::remove_dir_all(workspace.cache_dir()).await?;
        }
        self.workspace_provider
            .remove_isobin_manifest_dir_from_workspace_map(isobin_manifest_dir)
            .await
    }
}

#[derive(Getters)]
pub struct CleanServiceOptionBase<P> {
    quiet: bool,
    isobin_manifest_path: P,
}

pub type CleanServiceOption = CleanServiceOptionBase<Option<PathBuf>>;
type FiexedCleanServiceOption = CleanServiceOptionBase<PathBuf>;

impl CleanServiceOption {
    pub async fn fix(self) -> Result<FiexedCleanServiceOption> {
        let isobin_manifest_path =
            isobin_manifest_path_canonicalize(self.isobin_manifest_path).await?;
        Ok(FiexedCleanServiceOption {
            quiet: self.quiet,
            isobin_manifest_path,
        })
    }
}

#[derive(Default)]
pub struct CleanServiceOptionBuilder {
    quiet: bool,
    isobin_manifest_path: Option<PathBuf>,
}

impl CleanServiceOptionBuilder {
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }
    pub fn isobin_manifest_path(mut self, isobin_manifest_path: PathBuf) -> Self {
        self.isobin_manifest_path = Some(isobin_manifest_path);
        self
    }

    pub fn build(self) -> CleanServiceOption {
        CleanServiceOption {
            quiet: self.quiet,
            isobin_manifest_path: self.isobin_manifest_path,
        }
    }
}
