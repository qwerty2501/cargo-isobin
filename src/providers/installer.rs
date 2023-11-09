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
pub trait CoreInstaller: 'static + Send + Sync + Clone {
    type InstallTarget: InstallTarget;
    fn provider_kind(&self) -> providers::ProviderKind;
    fn multi_install_mode(&self) -> MultiInstallMode;
    async fn install(&self, target: &Self::InstallTarget) -> Result<()>;
    async fn uninstall(&self, target: &Self::InstallTarget) -> Result<()>;
}

pub enum MultiInstallMode {
    Parallel,
    Sequential,
}

#[derive(Clone)]
pub enum InstallTargetMode {
    Install,
    AlreadyInstalled,
    Uninstall,
}

pub trait InstallTarget: 'static + Send + Sync + Clone {
    fn mode(&self) -> &InstallTargetMode;
    fn provider_kind(&self) -> ProviderKind;
    fn name(&self) -> &str;
    fn summary(&self) -> String;
}

#[async_trait]
pub trait BinPathInstaller: 'static + Send + Sync + Clone {
    type InstallTarget: InstallTarget;
    async fn bin_paths(&self, target: &Self::InstallTarget) -> Result<Vec<PathBuf>>;
    async fn install_bin_path(&self, target: &Self::InstallTarget) -> Result<()>;
    async fn uninstall_bin_path(&self, target: &Self::InstallTarget) -> Result<()>;
}
