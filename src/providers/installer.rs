use super::*;
type Result<T> = std::result::Result<T, InstallError>;
#[async_trait]
pub trait Installer: 'static + Send + Sync {
    type InstallTarget: InstallTarget;

    fn provider_type(&self) -> &providers::ProviderType;
    async fn installs(&self, targets: &[Self::InstallTarget]) -> Result<()>;
    async fn install(&self, target: &Self::InstallTarget) -> Result<()> {
        target.install()
    }
}

#[async_trait]
pub trait InstallTarget: 'static + Send + Sync {
    fn provider_type(&self) -> &providers::ProviderType;

    fn install(&self) -> Result<()>;
}

#[derive(thiserror::Error, Debug, new)]
pub enum InstallError {
    #[error("todo")]
    Todo(),
}
