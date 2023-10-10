use std::path::PathBuf;

use tokio::process::Command;

use crate::{
    paths::workspace::Workspace,
    utils::{
        command_ext::run_install_command,
        fs_ext::{enumerate_executable_files, make_hard_links_in_dir},
    },
};

use super::*;

pub struct CargoInstallerFactory {
    cargo_workspace: CargoWorkspace,
    workspace: Workspace,
}

impl CargoInstallerFactory {
    pub fn new(workspace: Workspace) -> Self {
        Self {
            cargo_workspace: CargoWorkspace::from_workspace(&workspace),
            workspace,
        }
    }
}

#[async_trait]
impl InstallerFactory for CargoInstallerFactory {
    const INSTALLER_NAME: &'static str = CargoCoreInstaller::INSTALLER_NAME;
    type InstallTarget = CargoInstallTarget;
    type CoreInstaller = CargoCoreInstaller;
    type BinPathInstaller = CargoBinPathInstaller;
    async fn create_core_installer(&self) -> Result<Self::CoreInstaller> {
        Ok(CargoCoreInstaller::new(self.cargo_workspace.clone()))
    }
    async fn create_bin_path_installer(&self) -> Result<Self::BinPathInstaller> {
        Ok(CargoBinPathInstaller::new(
            self.cargo_workspace.clone(),
            self.workspace.clone(),
        ))
    }
}

#[derive(new)]
pub struct CargoCoreInstaller {
    cargo_workspace: CargoWorkspace,
}

impl CargoCoreInstaller {
    fn dependency_to_args(dependency: &CargoInstallDependencyDetail) -> Vec<String> {
        let mut args: Vec<String> = vec![];
        if let Some(version) = dependency.version() {
            args.extend_from_slice(&["--version".into(), version.into()]);
        }
        if let Some(registry) = dependency.registry() {
            args.extend_from_slice(&["--registry".into(), registry.into()]);
        }
        if let Some(index) = dependency.index() {
            args.extend_from_slice(&["--index".into(), index.into()]);
        }
        if let Some(path) = dependency.path() {
            args.extend_from_slice(&["--path".into(), path.into()]);
        }
        if let Some(git) = dependency.git() {
            args.extend_from_slice(&["--git".into(), git.into()]);
        }
        if let Some(branch) = dependency.branch() {
            args.extend_from_slice(&["--branch".into(), branch.into()]);
        }
        if let Some(tag) = dependency.tag() {
            args.extend_from_slice(&["--tag".into(), tag.into()]);
        }
        if let Some(rev) = dependency.rev() {
            args.extend_from_slice(&["--rev".into(), rev.into()]);
        }

        if let Some(bins) = dependency.bins() {
            for bin in bins.iter() {
                args.extend_from_slice(&["--bin".into(), bin.into()]);
            }
        }
        if let Some(features) = dependency.features() {
            args.extend_from_slice(&["--features".into(), features.join(",")]);
        }

        if let Some(all_features) = dependency.all_features() {
            if *all_features {
                args.push("--all-features".into());
            }
        }
        args
    }
}

#[async_trait]
impl providers::CoreInstaller for CargoCoreInstaller {
    const INSTALLER_NAME: &'static str = "cargo";
    type InstallTarget = CargoInstallTarget;

    fn provider_kind(&self) -> providers::ProviderKind {
        providers::ProviderKind::Cargo
    }
    fn multi_install_mode(&self) -> providers::MultiInstallMode {
        providers::MultiInstallMode::Parallel
    }

    async fn install(&self, target: &Self::InstallTarget) -> Result<()> {
        let mut command = Command::new(Self::INSTALLER_NAME);
        let mut args: Vec<String> = vec![
            "install".into(),
            "--force".into(),
            "--root".into(),
            self.cargo_workspace
                .cargo_home_dir()
                .to_string_lossy()
                .into(),
        ];
        let dependency = target.install_dependency().to_detail();
        args.extend_from_slice(&Self::dependency_to_args(&dependency));
        args.push(target.name().into());
        command.args(args);
        run_install_command(Self::INSTALLER_NAME, target.name(), command).await
    }
}

#[derive(new, Getters)]
pub struct CargoInstallTarget {
    name: String,
    install_dependency: CargoInstallDependency,
}

#[async_trait]
impl providers::InstallTarget for CargoInstallTarget {
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct CargoBinPathInstaller {
    cargo_workspace: CargoWorkspace,
    workspace: Workspace,
}

impl CargoBinPathInstaller {
    fn new(cargo_workspace: CargoWorkspace, workspace: Workspace) -> Self {
        Self {
            cargo_workspace,
            workspace,
        }
    }
}

#[async_trait]
impl BinPathInstaller for CargoBinPathInstaller {
    type InstallTarget = CargoInstallTarget;

    async fn bin_paths(&self) -> Result<Vec<PathBuf>> {
        Ok(enumerate_executable_files(self.cargo_workspace.cargo_bin_dir()).await?)
    }
    async fn install_bin_path(&self, _: &[Self::InstallTarget]) -> Result<()> {
        Ok(make_hard_links_in_dir(
            self.cargo_workspace.cargo_bin_dir(),
            self.workspace.bin_dir(),
        )
        .await?)
    }
}
