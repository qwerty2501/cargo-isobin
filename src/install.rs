use super::*;
use async_std::sync::Arc;

pub struct InstallRunnerProvider;

impl InstallRunnerProvider {
    pub fn make_runner<I: providers::Installer>(
        installer: I,
        targets: Vec<I::InstallTarget>,
    ) -> Arc<dyn InstallRunner> {
        Arc::new(InstallRunnerImpl::new(installer, targets))
    }
}

#[async_trait]
pub trait InstallRunner {
    fn provider_type(&self) -> providers::ProviderKind;
    async fn run_installs(&self) -> Result<()>;
}

#[derive(new)]
struct InstallRunnerImpl<I: providers::Installer> {
    installer: I,
    targets: Vec<I::InstallTarget>,
}

#[async_trait]
impl<I: providers::Installer> InstallRunner for InstallRunnerImpl<I> {
    fn provider_type(&self) -> providers::ProviderKind {
        self.installer.provider_type()
    }

    async fn run_installs(&self) -> Result<()> {
        Ok(self.installer.installs(&self.targets).await?)
    }
}

type Result<T> = std::result::Result<T, InstallRunError>;

#[derive(thiserror::Error, Debug, new)]
pub enum InstallRunError {
    #[error("{0}")]
    Install(#[from] providers::InstallError),
}
