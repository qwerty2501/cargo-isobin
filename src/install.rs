use crate::paths::isobin_config::IsobinConfigPathError;
use crate::paths::workspace::WorkspaceProvider;
use crate::providers::cargo::CargoConfig;
use crate::providers::cargo::CargoInstallTarget;
use crate::utils::fs_ext;
use crate::{paths::project::Project, providers::cargo::CargoInstallerFactory};
use async_std::path::PathBuf;
use std::collections::HashSet;

use super::*;
use async_std::sync::Arc;

#[derive(PartialEq)]
pub enum InstallMode {
    All,
    SpecificInstallTargetsOnly {
        specific_install_targets: Vec<String>,
    },
}

#[derive(new)]
pub struct InstallService {
    #[allow(dead_code)]
    project: Project,
    workspace_provider: WorkspaceProvider,
}

impl Default for InstallService {
    fn default() -> Self {
        let project = Project::default();
        Self {
            workspace_provider: WorkspaceProvider::new(project.clone()),
            project,
        }
    }
}

impl InstallService {
    #[allow(unused_variables)]
    pub async fn install(
        &self,
        service_option: &ServiceOption,
        install_service_option: &InstallServiceOption,
    ) -> Result<()> {
        let isobin_config = service_option.isobin_config();
        let isobin_config_dir = service_option
            .isobin_config_path()
            .parent()
            .ok_or(IsobinConfigPathError::NotFoundIsobinConfig)?;
        let workspace = self
            .workspace_provider
            .base_unique_workspace_dir_from_isobin_config_dir(&isobin_config_dir)
            .await?;
        let cargo_installer_factory = CargoInstallerFactory::new(workspace.clone());
        let cargo_runner = InstallRunnerProvider::make_cargo_runner(
            &cargo_installer_factory,
            isobin_config.cargo(),
        )
        .await?;
        fs_ext::clean_dir(workspace.bin_dir())
            .await
            .map_err(|e| Error::new_fatal(e.into()))?;
        self.run_each_installs(vec![cargo_runner]).await
    }

    async fn run_each_installs(&self, runners: Vec<Arc<dyn InstallRunner>>) -> Result<()> {
        await_futures!(runners.iter().map(|r| r.run_installs()))
            .map_err(InstallServiceError::MultiInstall)?;
        let mut keys = HashSet::new();
        let mut duplicates = vec![];
        for file_name in await_futures!(runners.iter().map(|r| r.bin_paths()))
            .map_err(InstallServiceError::MultiInstall)?
            .into_iter()
            .flatten()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
        {
            if keys.insert(file_name.clone()) {
                duplicates.push(file_name);
            }
        }
        if !duplicates.is_empty() {
            Err(InstallServiceError::new_duplicate_bin(duplicates).into())
        } else {
            await_futures!(runners.iter().map(|r| r.install_bin_path()))
                .map_err(InstallServiceError::MultiInstall)?;
            Ok(())
        }
    }
}

pub struct InstallRunnerProvider;

impl InstallRunnerProvider {
    pub async fn make_cargo_runner(
        cargo_installer: &CargoInstallerFactory,
        cargo_config: &CargoConfig,
    ) -> Result<Arc<dyn InstallRunner>> {
        let install_targets = cargo_config
            .installs()
            .iter()
            .map(|(name, install_dependency)| {
                CargoInstallTarget::new(name.into(), install_dependency.clone())
            })
            .collect::<Vec<_>>();
        Self::make_runner(cargo_installer, install_targets).await
    }

    async fn make_runner<IF: providers::InstallerFactory>(
        installer_factory: &IF,
        targets: Vec<IF::InstallTarget>,
    ) -> Result<Arc<dyn InstallRunner>> {
        let core_installer = installer_factory.create_core_installer().await?;
        let bin_path_installer = installer_factory.create_bin_path_installer().await?;
        Ok(Arc::new(InstallRunnerImpl::new(
            core_installer,
            bin_path_installer,
            targets,
        )))
    }
}

#[async_trait]
pub trait InstallRunner: 'static + Sync + Send {
    fn provider_type(&self) -> providers::ProviderKind;
    async fn run_installs(&self) -> Result<()>;
    async fn bin_paths(&self) -> Result<Vec<PathBuf>>;
    async fn install_bin_path(&self) -> Result<()>;
}

#[derive(new)]
struct InstallRunnerImpl<
    IT: providers::InstallTarget,
    CI: providers::CoreInstaller<InstallTarget = IT>,
    BI: providers::BinPathInstaller<InstallTarget = IT>,
> {
    core_installer: CI,
    bin_path_installer: BI,
    targets: Vec<IT>,
}

impl<
        IT: providers::InstallTarget,
        CI: providers::CoreInstaller<InstallTarget = IT>,
        BI: providers::BinPathInstaller<InstallTarget = IT>,
    > InstallRunnerImpl<IT, CI, BI>
{
    async fn run_sequential_installs(&self) -> Result<()> {
        for target in self.targets.iter() {
            self.core_installer.install(target).await?;
        }
        Ok(())
    }
    async fn run_parallel_installs(&self) -> Result<()> {
        let mut target_futures = Vec::with_capacity(self.targets.len());
        for target in self.targets.iter() {
            target_futures.push(self.core_installer.install(target));
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
impl<
        IT: providers::InstallTarget,
        CI: providers::CoreInstaller<InstallTarget = IT>,
        BI: providers::BinPathInstaller<InstallTarget = IT>,
    > InstallRunner for InstallRunnerImpl<IT, CI, BI>
{
    fn provider_type(&self) -> providers::ProviderKind {
        self.core_installer.provider_kind()
    }

    async fn run_installs(&self) -> Result<()> {
        match self.core_installer.multi_install_mode() {
            providers::MultiInstallMode::Parallel => self.run_parallel_installs().await,
            providers::MultiInstallMode::Sequential => self.run_sequential_installs().await,
        }
    }
    async fn bin_paths(&self) -> Result<Vec<PathBuf>> {
        self.bin_path_installer.bin_paths().await
    }
    async fn install_bin_path(&self) -> Result<()> {
        self.bin_path_installer
            .install_bin_path(&self.targets)
            .await
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

    #[error("duplicate bins:{0:#?}")]
    DuplicateBin(Vec<String>),
}
