use std::path::PathBuf;

use crate::{
    paths::{
        isobin_manifest::{isobin_manifest_dir, isobin_manifest_path_canonicalize},
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
        let isobin_manifest_dir = isobin_manifest_dir(path_service_option.isobin_manifest_path())?;
        let workspace = self
            .workspace_provider
            .base_unique_workspace_dir_from_isobin_manifest_dir(isobin_manifest_dir)
            .await?;
        Ok(workspace.bin_dir().into())
    }
}

#[derive(Getters)]
pub struct PathServiceOptionBase<P> {
    quiet: bool,
    isobin_manifest_path: P,
}

pub type PathServiceOption = PathServiceOptionBase<Option<PathBuf>>;
type FixedPathServiceOption = PathServiceOptionBase<PathBuf>;

impl PathServiceOption {
    async fn fix(self) -> Result<FixedPathServiceOption> {
        let isobin_manifest_path =
            isobin_manifest_path_canonicalize(self.isobin_manifest_path).await?;
        Ok(FixedPathServiceOption {
            quiet: self.quiet,
            isobin_manifest_path,
        })
    }
}

#[derive(Default)]
pub struct PathServiceOptionBuilder {
    quiet: bool,
    isobin_manifest_path: Option<PathBuf>,
}

impl PathServiceOptionBuilder {
    pub fn isobin_manifest_path(mut self, isobin_manifest_path: PathBuf) -> Self {
        self.isobin_manifest_path = Some(isobin_manifest_path);
        self
    }
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }
    pub fn build(self) -> PathServiceOption {
        PathServiceOption {
            quiet: self.quiet,
            isobin_manifest_path: self.isobin_manifest_path,
        }
    }
}
