use crate::bin_map::BinDependency;

use super::*;

#[async_trait]
pub trait InstallerFactory: 'static + Send + Sync {
    type InstallTarget: TargetDependency;
    type CoreInstaller: CoreInstaller<InstallTarget = Self::InstallTarget>;
    type BinPathInstaller: BinPathInstaller<InstallTarget = Self::InstallTarget>;
    async fn create_core_installer(&self) -> Result<Self::CoreInstaller>;
    async fn create_bin_path_installer(&self) -> Result<Self::BinPathInstaller>;
}

#[async_trait]
pub trait CoreInstaller: 'static + Send + Sync + Clone {
    type InstallTarget: TargetDependency;
    fn provider_kind(&self) -> providers::ProviderKind;
    fn multi_install_mode(&self) -> MultiInstallMode;
    async fn install(&self, target: &Self::InstallTarget) -> Result<()>;
    async fn uninstall(&self, target: &Self::InstallTarget) -> Result<()>;
}

pub enum MultiInstallMode {
    Parallel,
    #[allow(dead_code)]
    Sequential,
}

#[derive(Clone, PartialEq)]
pub enum TargetMode {
    Install,
    AlreadyInstalled,
    Uninstall,
}

pub trait TargetDependency: 'static + Send + Sync + Clone {
    fn mode(&self) -> &TargetMode;
    fn provider_kind(&self) -> ProviderKind;
    fn name(&self) -> &str;
    fn summary(&self) -> String;
}

#[derive(new, Getters, Clone)]
pub struct TargetBinDependency {
    mode: TargetMode,
    bin_dependency: BinDependency,
}

#[async_trait]
pub trait BinPathInstaller: 'static + Send + Sync + Clone {
    type InstallTarget: TargetDependency;
    async fn bin_paths(&self, target: &Self::InstallTarget) -> Result<Vec<TargetBinDependency>>;
    async fn install_bin_path(&self, target: &Self::InstallTarget) -> Result<()>;
    async fn uninstall_bin_path(&self, target: &Self::InstallTarget) -> Result<()>;
}
