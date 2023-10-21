use super::*;
use std::path::PathBuf;

#[async_trait]
pub trait InstallerFactory: 'static + Send + Sync {
    type InstallTarget: InstallTarget;
    type CoreInstaller: CoreInstaller<InstallTarget = Self::InstallTarget>;
    type BinPathInstaller: BinPathInstaller<InstallTarget = Self::InstallTarget>;
    async fn create_core_installer(&self) -> Result<Self::CoreInstaller>;
    async fn create_bin_path_installer(&self) -> Result<Self::BinPathInstaller>;
}

#[async_trait]
pub trait CoreInstaller: 'static + Send + Sync {
    type InstallTarget: InstallTarget;
    fn provider_kind(&self) -> providers::ProviderKind;
    fn multi_install_mode(&self) -> MultiInstallMode;
    async fn install(&self, target: &Self::InstallTarget) -> Result<()>;
}

pub enum MultiInstallMode {
    Parallel,
    Sequential,
}

pub trait InstallTarget: 'static + Send + Sync {
    fn provider_kind(&self) -> ProviderKind;
    fn name(&self) -> &str;
}

#[async_trait]
pub trait BinPathInstaller: 'static + Send + Sync {
    type InstallTarget: InstallTarget;
    async fn bin_paths(&self, targets: &[Self::InstallTarget]) -> Result<Vec<PathBuf>>;
    async fn install_bin_path(&self, targets: &[Self::InstallTarget]) -> Result<()>;
}
