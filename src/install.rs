use crate::paths::project::Project;

use super::*;
use async_std::sync::Arc;

#[derive(PartialEq)]
pub enum InstallMode {
    All,
    SpecificInstallTargetsOnly {
        specific_install_targets: Vec<String>,
    },
}

#[derive(new, Default)]
pub struct InstallService {
    #[allow(dead_code)]
    project: Project,
}

impl InstallService {
    #[allow(unused_variables)]
    pub async fn install(
        &self,
        service_option: &ServiceOption,
        install_service_option: &InstallServiceOption,
    ) -> Result<()> {
        let isobin_config_path = service_option.isobin_config();
        todo!()
    }
}

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

#[derive(Getters)]
pub struct InstallServiceOption {
    mode: InstallMode,
}

pub struct InstallServiceOptionBuilder {
    mode: Option<InstallMode>,
}

impl InstallServiceOptionBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { mode: None }
    }
    pub fn mode(self, mode: InstallMode) -> Self {
        InstallServiceOptionBuilder { mode: Some(mode) }
    }
    pub fn build(self) -> InstallServiceOption {
        InstallServiceOption {
            mode: self.mode.unwrap_or(InstallMode::All),
        }
    }
}

type Result<T> = std::result::Result<T, InstallServiceError>;

#[derive(thiserror::Error, Debug, new)]
pub enum InstallServiceError {
    #[error("{0}")]
    Install(#[from] providers::InstallError),
}
