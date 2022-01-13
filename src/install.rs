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

impl<I: providers::Installer> InstallRunnerImpl<I> {
    async fn run_sequential_installs(&self) -> Result<()> {
        for target in self.targets.iter() {
            self.installer.install(target).await?;
        }
        Ok(())
    }
    async fn run_parallel_installs(&self) -> Result<()> {
        let mut target_futures = Vec::with_capacity(self.targets.len());
        for target in self.targets.iter() {
            target_futures.push(self.installer.install(target));
        }
        let mut target_errors = vec![];
        for target_future in target_futures.into_iter() {
            let result = target_future.await;
            if let Some(err) = result.err() {
                target_errors.push(err);
            }
        }
        if !target_errors.is_empty() {
            Err(InstallServiceError::MultiInstall(target_errors))
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<I: providers::Installer> InstallRunner for InstallRunnerImpl<I> {
    fn provider_type(&self) -> providers::ProviderKind {
        self.installer.provider_kind()
    }

    async fn run_installs(&self) -> Result<()> {
        match self.installer.multi_install_mode() {
            providers::MultiInstallMode::Parallel => self.run_parallel_installs().await,
            providers::MultiInstallMode::Sequential => self.run_sequential_installs().await,
        }
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

    #[error("occurred multi error")]
    MultiInstall(Vec<providers::InstallError>),
}
