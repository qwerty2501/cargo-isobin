use super::*;
#[async_trait]
pub trait Installer: 'static + Send + Sync {
    type InstallTarget: InstallTarget;

    fn provider_kind(&self) -> providers::ProviderKind;
    fn multi_install_mode(&self) -> MultiInstallMode;
    async fn install(&self, target: &Self::InstallTarget) -> Result<()>;
}

pub enum MultiInstallMode {
    Parallel,
    Sequential,
}

#[async_trait]
pub trait InstallTarget: 'static + Send + Sync {}
