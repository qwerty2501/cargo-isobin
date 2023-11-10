use nanoid::nanoid;
use tokio::fs;
use tokio::sync::Mutex;

use crate::fronts::MultiProgress;
use crate::fronts::Progress;
use crate::paths::isobin_manifest::isobin_manifest_dir;
use crate::paths::isobin_manifest::isobin_manifest_path_canonicalize;
use crate::paths::workspace::Workspace;
use crate::paths::workspace::WorkspaceProvider;
use crate::providers::cargo::CargoConfig;
use crate::providers::cargo::CargoInstallTarget;
use crate::providers::cargo::CargoInstallerFactory;
use crate::providers::InstallTarget;
use crate::providers::InstallTargetMode;
use crate::utils::fs_ext;
use crate::utils::fs_ext::copy_dir;
use std::collections::HashSet;
use std::path::PathBuf;

use super::*;
use std::sync::Arc;

#[derive(PartialEq)]
pub enum InstallMode {
    All,
    SpecificInstallTargetsOnly {
        specific_install_targets: Vec<String>,
    },
}

#[derive(Default)]
pub struct InstallService {
    workspace_provider: WorkspaceProvider,
}

impl InstallService {
    pub async fn install<MP: MultiProgress>(
        &self,
        install_service_option: InstallServiceOption,
    ) -> Result<()> {
        let install_service_option = install_service_option.fix().await?;
        let isobin_manifest =
            IsobinManifest::load_from_file(install_service_option.isobin_manifest_path()).await?;
        let isobin_manifest_dir =
            isobin_manifest_dir(install_service_option.isobin_manifest_path())?;
        let workspace = self
            .workspace_provider
            .base_unique_workspace_dir_from_isobin_manifest_dir(isobin_manifest_dir)
            .await?;
        let tmp_workspace = workspace.make_tmp_workspace();
        if workspace.base_dir().exists() {
            fs_ext::create_dir_if_not_exists(tmp_workspace.base_dir()).await?;
            copy_dir(
                workspace.base_dir().clone(),
                tmp_workspace.base_dir().clone(),
            )
            .await?;
        }
        let isobin_manifest_cache = if install_service_option.force {
            IsobinManifest::default()
        } else {
            IsobinConfigCache::lenient_load_cache_from_dir(tmp_workspace.base_dir()).await
        };
        let specified_isobin_manifest = match install_service_option.mode() {
            InstallMode::All => isobin_manifest,
            InstallMode::SpecificInstallTargetsOnly {
                specific_install_targets,
            } => isobin_manifest.filter_target(specific_install_targets),
        };
        let install_target_isobin_manifest = IsobinManifest::get_need_install_config(
            &specified_isobin_manifest,
            &isobin_manifest_cache,
            &tmp_workspace,
        )
        .await?;

        let save_isobin_manifest =
            IsobinManifest::merge(&isobin_manifest_cache, &specified_isobin_manifest);

        self.run_install::<MP>(
            &workspace,
            &tmp_workspace,
            &save_isobin_manifest,
            &specified_isobin_manifest,
            &install_target_isobin_manifest,
            &IsobinManifest::default(),
        )
        .await
    }

    async fn run_install<MP: MultiProgress>(
        &self,
        workspace: &Workspace,
        tmp_workspace: &Workspace,
        save_isobin_manifest: &IsobinManifest,
        specified_isobin_manifest: &IsobinManifest,
        install_target_isobin_manifest: &IsobinManifest,
        uninstall_target_isobin_manifest: &IsobinManifest,
    ) -> Result<()> {
        fs_ext::create_dir_if_not_exists(tmp_workspace.base_dir()).await?;
        IsobinConfigCache::save_cache_to_dir(save_isobin_manifest, tmp_workspace.base_dir())
            .await?;

        let cargo_installer_factory = CargoInstallerFactory::new(tmp_workspace.clone());
        let install_runner_provider = InstallRunnerProvider::<MP>::default();
        let cargo_runner = install_runner_provider
            .make_cargo_runner(
                &cargo_installer_factory,
                specified_isobin_manifest.cargo(),
                install_target_isobin_manifest.cargo(),
                uninstall_target_isobin_manifest.cargo(),
            )
            .await?;
        self.run_each_installs(workspace, tmp_workspace, vec![cargo_runner])
            .await
    }

    async fn run_each_installs(
        &self,
        workspace: &Workspace,
        tmp_workspace: &Workspace,
        runners: Vec<Arc<Mutex<dyn InstallRunner>>>,
    ) -> Result<()> {
        let install_runners = runners.clone();
        join_futures!(install_runners
            .into_iter()
            .map(|r| async move { r.lock().await.run_installs().await }))
        .await
        .map_err(InstallServiceError::MultiInstall)?;
        let mut keys = HashSet::new();
        let mut duplicates = vec![];
        let file_name_runners = runners.clone();
        for file_name in join_futures!(file_name_runners
            .into_iter()
            .map(|r| async move { r.lock().await.bin_paths().await }))
        .await
        .map_err(InstallServiceError::MultiInstall)?
        .into_iter()
        .flatten()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
        {
            if !keys.insert(file_name.clone()) {
                duplicates.push(file_name);
            }
        }
        if !duplicates.is_empty() {
            Err(InstallServiceError::new_duplicate_bin(duplicates).into())
        } else {
            let install_bin_path_runners = runners.clone();
            join_futures!(install_bin_path_runners
                .into_iter()
                .map(|r| async move { r.lock().await.install_bin_path().await }))
            .await
            .map_err(InstallServiceError::MultiInstall)?;

            let tmp_dir = workspace.cache_dir().join(nanoid!());
            let need_tmp = workspace.base_dir().exists();
            if need_tmp {
                fs::rename(workspace.base_dir(), &tmp_dir).await?;
            }
            match fs::rename(tmp_workspace.base_dir(), workspace.base_dir()).await {
                Ok(_) => {}
                Err(err) => {
                    if need_tmp {
                        fs::rename(&tmp_dir, workspace.base_dir()).await?;
                    }
                    Err(err)?;
                }
            }
            if need_tmp {
                fs_ext::clean_dir(tmp_dir).await?
            }
            Ok(())
        }
    }
}

#[derive(Getters, new, Clone)]
struct InstallTargetContext<IF: InstallTarget + Clone, P: Progress> {
    target: IF,
    progress: P,
}

#[derive(Default)]
pub struct InstallRunnerProvider<MP: MultiProgress> {
    multi_progress: MP,
}

impl<MP: MultiProgress> InstallRunnerProvider<MP> {
    pub async fn make_cargo_runner(
        &self,
        cargo_installer: &CargoInstallerFactory,
        specified_cargo_config: &CargoConfig,
        install_target_cargo_config: &CargoConfig,
        uninstall_target_cargo_config: &CargoConfig,
    ) -> Result<Arc<Mutex<dyn InstallRunner>>> {
        let install_targets = specified_cargo_config
            .installs()
            .iter()
            .map(|(name, install_dependency)| {
                let mode = if install_target_cargo_config.installs().get(name).is_some() {
                    InstallTargetMode::Install
                } else if uninstall_target_cargo_config.installs().get(name).is_some() {
                    InstallTargetMode::Uninstall
                } else {
                    InstallTargetMode::AlreadyInstalled
                };
                CargoInstallTarget::new(name.into(), install_dependency.clone(), mode)
            })
            .collect::<Vec<_>>();
        self.make_runner(cargo_installer, install_targets).await
    }

    async fn make_runner<IF: providers::InstallerFactory>(
        &self,
        installer_factory: &IF,
        targets: Vec<IF::InstallTarget>,
    ) -> Result<Arc<Mutex<dyn InstallRunner>>> {
        let core_installer = installer_factory.create_core_installer().await?;
        let bin_path_installer = installer_factory.create_bin_path_installer().await?;
        let contexts = targets
            .into_iter()
            .map(|target| {
                let progress = self.multi_progress.make_progress(&target);
                let context = InstallTargetContext::new(target, progress);
                context.progress().prepare_install()?;
                Ok(context)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Arc::new(Mutex::new(InstallRunnerImpl::new(
            core_installer,
            bin_path_installer,
            contexts,
        ))))
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
    P: Progress,
> {
    core_installer: CI,
    bin_path_installer: BI,
    contexts: Vec<InstallTargetContext<IT, P>>,
}

impl<
        IT: providers::InstallTarget,
        CI: providers::CoreInstaller<InstallTarget = IT>,
        BI: providers::BinPathInstaller<InstallTarget = IT>,
        P: Progress,
    > InstallRunnerImpl<IT, CI, BI, P>
{
    async fn run_sequential_installs(&self) -> Result<()> {
        for context in self.contexts.iter() {
            Self::install(
                self.core_installer.clone(),
                self.bin_path_installer.clone(),
                context.clone(),
            )
            .await?;
        }
        Ok(())
    }
    async fn run_parallel_installs(&self) -> Result<()> {
        join_futures!(self.contexts.iter().map(|target| {
            Self::install(
                self.core_installer.clone(),
                self.bin_path_installer.clone(),
                target.clone(),
            )
        }))
        .await
        .map_err(InstallServiceError::MultiInstall)?;
        Ok(())
    }
    async fn install(
        core_installer: CI,
        bin_path_installer: BI,
        install_context: InstallTargetContext<IT, P>,
    ) -> Result<()> {
        let progress = install_context.progress();
        let target = install_context.target();
        match target.mode() {
            InstallTargetMode::Install => {
                progress.start_install()?;
                match core_installer.install(target).await {
                    Ok(_) => {
                        progress.done_install()?;
                        Ok(())
                    }
                    Err(err) => {
                        progress.failed_install()?;
                        Err(err)
                    }
                }
            }
            InstallTargetMode::AlreadyInstalled => progress.already_installed(),
            InstallTargetMode::Uninstall => {
                progress.start_uninstall()?;
                match Self::uninstall(core_installer, bin_path_installer, target).await {
                    Ok(_) => {
                        progress.done_uninstall()?;
                        Ok(())
                    }
                    Err(err) => {
                        progress.failed_uninstall()?;
                        Err(err)
                    }
                }
            }
        }
    }
    async fn uninstall(core_installer: CI, bin_path_installer: BI, target: &IT) -> Result<()> {
        bin_path_installer.uninstall_bin_path(target).await?;
        core_installer.uninstall(target).await
    }
}

#[async_trait]
impl<
        IT: providers::InstallTarget,
        CI: providers::CoreInstaller<InstallTarget = IT>,
        BI: providers::BinPathInstaller<InstallTarget = IT>,
        P: Progress,
    > InstallRunner for InstallRunnerImpl<IT, CI, BI, P>
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
        let bin_paths = join_futures!(self.contexts.iter().map(|context| {
            let bin_path_installer = self.bin_path_installer.clone();
            let target = context.target().clone();
            async move { bin_path_installer.bin_paths(&target).await }
        }))
        .await
        .map_err(InstallServiceError::MultiInstall)?;
        Ok(bin_paths.into_iter().flatten().collect())
    }
    async fn install_bin_path(&self) -> Result<()> {
        join_futures!(self.contexts.iter().map(|context| {
            let bin_path_installer = self.bin_path_installer.clone();
            let target = context.target().clone();
            async move { bin_path_installer.install_bin_path(&target).await }
        }))
        .await
        .map_err(InstallServiceError::MultiInstall)?;
        Ok(())
    }
}

#[derive(Getters)]
pub struct InstallServiceOption {
    quiet: bool,
    force: bool,
    mode: InstallMode,
    isobin_manifest_path: Option<PathBuf>,
}

#[derive(Getters)]
pub struct FixedInstallServiceOption {
    quiet: bool,
    force: bool,
    mode: InstallMode,
    isobin_manifest_path: PathBuf,
}

impl InstallServiceOption {
    pub async fn fix(self) -> Result<FixedInstallServiceOption> {
        let isobin_manifest_path =
            isobin_manifest_path_canonicalize(self.isobin_manifest_path).await?;
        Ok(FixedInstallServiceOption {
            quiet: self.quiet,
            force: self.force,
            mode: self.mode,
            isobin_manifest_path,
        })
    }
}

#[derive(Default)]
pub struct InstallServiceOptionBuilder {
    quiet: bool,
    force: bool,
    mode: Option<InstallMode>,
    isobin_manifest_path: Option<PathBuf>,
}

impl InstallServiceOptionBuilder {
    pub fn mode(mut self, mode: InstallMode) -> Self {
        self.mode = Some(mode);
        self
    }
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
    pub fn isobin_manifest_path(mut self, isobin_manifest_path: PathBuf) -> Self {
        self.isobin_manifest_path = Some(isobin_manifest_path);
        self
    }

    pub fn build(self) -> InstallServiceOption {
        InstallServiceOption {
            quiet: self.quiet,
            force: self.force,
            mode: self.mode.unwrap_or(InstallMode::All),
            isobin_manifest_path: self.isobin_manifest_path,
        }
    }
}

#[derive(thiserror::Error, Debug, new)]
pub enum InstallServiceError {
    #[error("{0:#?}")]
    MultiInstall(Vec<Error>),

    #[error("{provider}/{name}:\n{error_message}")]
    Install {
        provider: String,
        name: String,
        error_message: String,
        error: Error,
    },

    #[error("duplicate bins:\n{0:#?}")]
    DuplicateBin(Vec<String>),
}
