use super::*;
use async_std::sync::Arc;

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

pub struct InstallRunnerProvider;

impl InstallRunnerProvider {
    pub fn make_runner<I: Installer>(
        installer: I,
        targets: Vec<I::InstallTarget>,
    ) -> Arc<dyn InstallRunner> {
        Arc::new(InstallRunnerImpl::new(installer, targets))
    }
}

#[async_trait]
pub trait InstallRunner {
    fn provider_type(&self) -> &providers::ProviderType;
    async fn run_installs(&self) -> Result<()>;
}

#[derive(new)]
struct InstallRunnerImpl<I: Installer> {
    installer: I,
    targets: Vec<I::InstallTarget>,
}

#[async_trait]
impl<I: Installer> InstallRunner for InstallRunnerImpl<I> {
    fn provider_type(&self) -> &providers::ProviderType {
        self.installer.provider_type()
    }

    async fn run_installs(&self) -> Result<()> {
        self.installer.installs(&self.targets).await
    }
}

#[derive(thiserror::Error, Debug, new)]
pub enum InstallError {
    #[error("todo")]
    Todo(),
}
