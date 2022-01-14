use super::*;
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

pub trait InstallTarget: 'static + Send + Sync {}

#[async_trait]
pub trait BinPathInstaller: 'static + Send + Sync {
    type InstallTarget: InstallTarget;
    async fn bin_names(&self) -> Vec<String>;
    async fn bin_installs(&self, targets: &[Self::InstallTarget]) -> Result<()>;
}
