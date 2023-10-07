use std::path::PathBuf;

use crate::{
    paths::workspace::Workspace,
    utils::fs_ext::{enumerate_executable_files, make_hard_links_in_dir},
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
    type InstallTarget = CargoInstallTarget;
    type CoreInstaller = CargoCoreInstaller;
    type BinPathInstaller = CargoBinPathInstaller;
    async fn create_core_installer(&self) -> Result<Self::CoreInstaller> {
        Ok(CargoCoreInstaller {})
    }
    async fn create_bin_path_installer(&self) -> Result<Self::BinPathInstaller> {
        Ok(CargoBinPathInstaller::new(
            self.cargo_workspace.clone(),
            self.workspace.clone(),
        ))
    }
}

pub struct CargoCoreInstaller {}

#[async_trait]
impl providers::CoreInstaller for CargoCoreInstaller {
    type InstallTarget = CargoInstallTarget;

    fn provider_kind(&self) -> providers::ProviderKind {
        providers::ProviderKind::Cargo
    }
    fn multi_install_mode(&self) -> providers::MultiInstallMode {
        providers::MultiInstallMode::Parallel
    }

    #[allow(unused_variables)]
    async fn install(&self, target: &Self::InstallTarget) -> Result<()> {
        todo!()
    }
}

#[derive(new, Getters)]
pub struct CargoInstallTarget {
    name: String,
    install_dependency: CargoInstallDependency,
}

#[async_trait]
impl providers::InstallTarget for CargoInstallTarget {}

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
        enumerate_executable_files(self.cargo_workspace.cargo_bin_dir())
            .await
            .map_err(|e| Error::new_fatal(e.into()))
    }
    async fn install_bin_path(&self, _: &[Self::InstallTarget]) -> Result<()> {
        make_hard_links_in_dir(
            self.cargo_workspace.cargo_bin_dir(),
            self.workspace.bin_dir(),
        )
        .await
        .map_err(|e| Error::new_fatal(e.into()))
    }
}
