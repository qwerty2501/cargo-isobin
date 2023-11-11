use std::path::PathBuf;

use tokio::{fs, process::Command};

use crate::{
    paths::workspace::Workspace,
    utils::{
        command_ext::{run_commnad, RunCommandError},
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

#[derive(new, Clone)]
pub struct CargoCoreInstaller {
    cargo_workspace: CargoWorkspace,
}

impl CargoCoreInstaller {
    fn dependency_to_args(dependency: &CargoInstallDependencyDetail) -> Vec<String> {
        let mut args: Vec<String> = vec![];
        if let Some(version) = dependency.version() {
            args.extend_from_slice(&["--version".into(), version.to_string()]);
        }
        if let Some(registry) = dependency.registry() {
            args.extend_from_slice(&["--registry".into(), registry.into()]);
        }
        if let Some(index) = dependency.index() {
            args.extend_from_slice(&["--index".into(), index.into()]);
        }
        if let Some(absolute_path) = dependency.absolute_path() {
            args.extend_from_slice(&["--path".into(), absolute_path.to_string_lossy().into()]);
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
    type InstallTarget = CargoInstallTarget;
    fn provider_kind(&self) -> providers::ProviderKind {
        providers::ProviderKind::Cargo
    }
    fn multi_install_mode(&self) -> providers::MultiInstallMode {
        providers::MultiInstallMode::Parallel
    }

    async fn install(&self, target: &Self::InstallTarget) -> Result<()> {
        let install_dir = self.cargo_workspace.cargo_home_dir().join(target.name());
        let mut command = Command::new(PROVIDER_NAME);
        let mut args: Vec<String> = vec![
            "--quiet".into(),
            "install".into(),
            "--force".into(),
            "--root".into(),
            install_dir.to_string_lossy().into(),
        ];
        let dependency_args = match target.install_dependency() {
            CargoInstallDependency::Simple(version) => Self::dependency_to_args(
                &CargoInstallDependencyDetail::from_version(version.clone()),
            ),
            CargoInstallDependency::Detailed(dependency) => Self::dependency_to_args(dependency),
        };
        args.extend_from_slice(&dependency_args);
        args.push(target.name().into());
        command.args(args);
        run_commnad(command)
            .await
            .map_err(|err| match err.downcast::<RunCommandError>() {
                Ok(err) => InstallServiceError::new_install(
                    ProviderKind::Cargo,
                    target.name().into(),
                    err.stderr().into(),
                    err.into(),
                )
                .into(),
                Err(err) => InstallServiceError::new_install(
                    ProviderKind::Cargo,
                    target.name().into(),
                    err.to_string(),
                    err,
                )
                .into(),
            })
    }

    async fn uninstall(&self, target: &Self::InstallTarget) -> Result<()> {
        let install_dir = self.cargo_workspace.cargo_home_dir().join(target.name());
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).await?;
        }
        Ok(())
    }
}

#[derive(new, Getters, Clone)]
pub struct CargoInstallTarget {
    name: String,
    install_dependency: CargoInstallDependency,
    mode: InstallTargetMode,
}

impl providers::InstallTarget for CargoInstallTarget {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Cargo
    }
    fn name(&self) -> &str {
        &self.name
    }

    fn mode(&self) -> &InstallTargetMode {
        &self.mode
    }
    fn summary(&self) -> String {
        match self.install_dependency() {
            CargoInstallDependency::Simple(version) => version.to_string(),
            CargoInstallDependency::Detailed(dependency) => {
                if let Some(git) = dependency.git() {
                    if let Some(rev) = dependency.rev() {
                        format!("{git} {rev}")
                    } else if let Some(version) = dependency.version() {
                        format!("{git} {version}")
                    } else {
                        git.to_string()
                    }
                } else if let Some(version) = dependency.version() {
                    version.to_string()
                } else if let Some(path) = dependency.path() {
                    path.to_string_lossy().to_string()
                } else {
                    String::new()
                }
            }
        }
    }
}
#[derive(Clone)]
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
    fn bin_dir(&self, target: &CargoInstallTarget) -> PathBuf {
        self.cargo_workspace
            .cargo_home_dir()
            .join(target.name())
            .join("bin")
    }
}

#[async_trait]
impl BinPathInstaller for CargoBinPathInstaller {
    type InstallTarget = CargoInstallTarget;

    async fn bin_paths(&self, target: &Self::InstallTarget) -> Result<Vec<PathBuf>> {
        enumerate_executable_files(self.bin_dir(target)).await
    }
    async fn install_bin_path(&self, target: &Self::InstallTarget) -> Result<()> {
        let cargo_bin_dir = self.bin_dir(target);
        make_hard_links_in_dir(cargo_bin_dir, self.workspace.bin_dir()).await?;
        Ok(())
    }

    async fn uninstall_bin_path(&self, target: &Self::InstallTarget) -> Result<()> {
        let bin_paths = self.bin_paths(target).await?;
        for bin_path in bin_paths.iter() {
            if let Some(file_name) = bin_path.file_name().map(|f| f.to_str().unwrap()) {
                let workspace_bin_path = self.workspace.bin_dir().join(file_name);
                if workspace_bin_path.exists() {
                    fs::remove_file(workspace_bin_path).await?;
                }
            }
        }
        Ok(())
    }
}
