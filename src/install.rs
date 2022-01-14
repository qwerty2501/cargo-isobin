use crate::providers::cargo::CargoConfig;
use crate::providers::cargo::CargoInstallTarget;
use crate::{paths::project::Project, providers::cargo::CargoInstaller};

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
        let isobin_config = service_option.isobin_config();
        let cargo_installer = CargoInstaller::default();
        let cargo_runner = InstallRunnerProvider::make_cargo_runner(
            CargoInstaller::default(),
            isobin_config.cargo(),
        );
        self.run_each_installs(vec![cargo_runner]).await
    }

    async fn run_each_installs(&self, runners: Vec<Arc<dyn InstallRunner>>) -> Result<()> {
        await_futures!(runners.iter().map(|r| r.run_installs()))
            .map_err(InstallServiceError::MultiInstall)?;
        Ok(())
    }
}

pub struct InstallRunnerProvider;

impl InstallRunnerProvider {
    pub fn make_cargo_runner(
        cargo_installer: CargoInstaller,
        cargo_config: &CargoConfig,
    ) -> Arc<dyn InstallRunner> {
        let install_targets = cargo_config
            .installs()
            .iter()
            .map(|(name, install_dependency)| {
                CargoInstallTarget::new(name.into(), install_dependency.clone())
            })
            .collect::<Vec<_>>();
        Self::make_runner(cargo_installer, install_targets)
    }

    fn make_runner<I: providers::CoreInstaller>(
        installer: I,
        targets: Vec<I::InstallTarget>,
    ) -> Arc<dyn InstallRunner> {
        Arc::new(InstallRunnerImpl::new(installer, targets))
    }
}

#[async_trait]
pub trait InstallRunner: 'static + Sync + Send {
    fn provider_type(&self) -> providers::ProviderKind;
    async fn run_installs(&self) -> Result<()>;
}

#[derive(new)]
struct InstallRunnerImpl<I: providers::CoreInstaller> {
    installer: I,
    targets: Vec<I::InstallTarget>,
}

impl<I: providers::CoreInstaller> InstallRunnerImpl<I> {
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
            Err(InstallServiceError::MultiInstall(target_errors).into())
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<I: providers::CoreInstaller> InstallRunner for InstallRunnerImpl<I> {
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

#[derive(thiserror::Error, Debug, new)]
pub enum InstallServiceError {
    #[error("occurred multi error")]
    MultiInstall(Vec<Error>),
}
