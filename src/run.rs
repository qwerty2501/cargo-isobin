use std::{
    path::PathBuf,
    process::{ExitStatus, Stdio},
};

use tokio::process::Command;

use crate::{
    bin_map::BinMap,
    flex_eprintln,
    manifest::{IsobinManifest, IsobinManifestCache},
    paths::{
        isobin_manifest::{isobin_manifest_dir, isobin_manifest_path_canonicalize},
        workspace::{Workspace, WorkspaceProvider},
    },
    InstallMode, Result, SpecifiedTarget,
};

#[derive(Default)]
pub struct RunService {
    workspace_provider: WorkspaceProvider,
}

impl RunService {
    pub async fn run(&self, run_service_option: RunServiceOption) -> Result<()> {
        let run_service_option = run_service_option.fix().await?;
        let isobin_manifest_dir = isobin_manifest_dir(run_service_option.isobin_manifest_path())?;
        let isobin_manifest =
            IsobinManifest::load_from_file(run_service_option.isobin_manifest_path()).await?;
        let workspace = self
            .workspace_provider
            .base_unique_workspace_dir_from_isobin_manifest_dir(isobin_manifest_dir)
            .await?;
        let bin_map = BinMap::lenient_load_from_dir(workspace.base_dir()).await?;
        if let Some(bin_dependency) = bin_map.bin_dependencies().get(run_service_option.bin()) {
            if isobin_manifest.exists_name(bin_dependency.name()) {
                let isobin_cache =
                    IsobinManifestCache::lenient_load_cache_from_dir(workspace.base_dir()).await;
                if isobin_manifest
                    .ditect_difference(
                        &isobin_cache,
                        bin_dependency.provider_kind(),
                        bin_dependency.name(),
                        &workspace,
                    )
                    .await?
                {
                    self.install_and_run(
                        &workspace,
                        SpecifiedTarget::new(
                            Some(bin_dependency.provider_kind().clone()),
                            bin_dependency.name().to_string(),
                        ),
                        run_service_option,
                    )
                    .await
                } else {
                    self.run_command(&workspace, run_service_option).await
                }
            } else {
                Err(RunServiceError::new_not_found_bin_dependency(
                    run_service_option.bin().to_string(),
                )
                .into())
            }
        } else if isobin_manifest.exists_name(run_service_option.bin()) {
            self.install_and_run(
                &workspace,
                SpecifiedTarget::new(None, run_service_option.bin().to_string()),
                run_service_option,
            )
            .await
        } else {
            Err(
                RunServiceError::new_not_found_bin_dependency(run_service_option.bin().to_string())
                    .into(),
            )
        }
    }
    async fn install_and_run(
        &self,
        workspace: &Workspace,
        specified_target: SpecifiedTarget,
        run_service_option: FixedRunServiceOption,
    ) -> Result<()> {
        flex_eprintln!(
            run_service_option.quiet,
            "ditected difference from current manifest"
        );
        crate::install(
            crate::InstallServiceOptionBuilder::default()
                .isobin_manifest_path(run_service_option.isobin_manifest_path().into())
                .mode(InstallMode::SpecificInstallTargetsOnly {
                    specified_install_targets: vec![specified_target],
                })
                .quiet(*run_service_option.quiet())
                .build(),
        )
        .await?;
        self.run_command(workspace, run_service_option).await
    }

    async fn run_command(
        &self,
        workspace: &Workspace,
        run_service_option: FixedRunServiceOption,
    ) -> Result<()> {
        let bin_file_path = workspace.bin_dir().join(run_service_option.bin());
        if bin_file_path.exists() && bin_file_path.is_file() {
            let status = Command::new(bin_file_path)
                .args(run_service_option.args())
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .await?;
            if status.success() {
                Ok(())
            } else {
                Err(RunServiceError::new_run_failed(status).into())
            }
        } else {
            Err(
                RunServiceError::new_not_found_bin_file(run_service_option.bin().to_string())
                    .into(),
            )
        }
    }
}

#[derive(thiserror::Error, Debug, new)]
pub enum RunServiceError {
    #[error("not found {bin} dependency in isobin manifest")]
    NotFoundBinDependency { bin: String },
    #[error("not found {bin} file in bin dir")]
    NotFoundBinFile { bin: String },
    #[error("")]
    RunFailed { status: ExitStatus },
}

#[derive(Getters)]
pub struct RunServiceOptionBase<P> {
    quiet: bool,
    bin: String,
    args: Vec<String>,
    isobin_manifest_path: P,
}

pub type RunServiceOption = RunServiceOptionBase<Option<PathBuf>>;

pub type FixedRunServiceOption = RunServiceOptionBase<PathBuf>;

impl RunServiceOption {
    pub async fn fix(self) -> Result<FixedRunServiceOption> {
        let isobin_manifest_path =
            isobin_manifest_path_canonicalize(self.isobin_manifest_path).await?;
        Ok(FixedRunServiceOption {
            quiet: self.quiet,
            bin: self.bin,
            args: self.args,
            isobin_manifest_path,
        })
    }
}

#[derive(Default)]
pub struct RunServiceOptionBuilder {
    quiet: bool,
    bin: String,
    args: Vec<String>,
    isobin_manifest_path: Option<PathBuf>,
}

impl RunServiceOptionBuilder {
    pub fn bin(mut self, bin: String) -> Self {
        self.bin = bin;
        self
    }
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }
    pub fn isobin_manifest_path(mut self, isobin_manifest_path: PathBuf) -> Self {
        self.isobin_manifest_path = Some(isobin_manifest_path);
        self
    }
    pub fn build(self) -> RunServiceOption {
        RunServiceOption {
            quiet: self.quiet,
            bin: self.bin,
            args: self.args,
            isobin_manifest_path: self.isobin_manifest_path,
        }
    }
}
