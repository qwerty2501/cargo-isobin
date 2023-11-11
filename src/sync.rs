use std::path::PathBuf;

use crate::{
    manifest::{IsobinManifest, IsobinManifestCache},
    paths::{
        isobin_manifest::{isobin_manifest_dir, isobin_manifest_path_canonicalize},
        workspace::WorkspaceProvider,
    },
    utils::fs_ext::{self, copy_dir},
    InstallService, Result,
};

#[derive(Default)]
pub struct SyncService {
    install_service: InstallService,
    workspace_provider: WorkspaceProvider,
}

impl SyncService {
    pub async fn sync(&self, sync_service_option: SyncServiceOption) -> Result<()> {
        let sync_service_option = sync_service_option.fix().await?;

        let isobin_manifest =
            IsobinManifest::load_from_file(sync_service_option.isobin_manifest_path()).await?;
        let isobin_manifest_dir = isobin_manifest_dir(sync_service_option.isobin_manifest_path())?;

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
        let isobin_manifest_cache = if sync_service_option.force {
            IsobinManifest::default()
        } else {
            IsobinManifestCache::lenient_load_cache_from_dir(tmp_workspace.base_dir()).await
        };

        let specified_isobin_manifest = isobin_manifest;
        let install_target_isobin_manifest = IsobinManifest::get_need_install_dependency_manifest(
            &specified_isobin_manifest,
            &isobin_manifest_cache,
            &tmp_workspace,
        )
        .await?;

        let uninstall_target_isobin_manifest =
            IsobinManifest::get_need_uninstall_dependency_manifest(
                &specified_isobin_manifest,
                &isobin_manifest_cache,
            )
            .await?;

        let save_isobin_manifest = isobin_manifest_cache.merge(&specified_isobin_manifest);
        let save_isobin_manifest =
            save_isobin_manifest.remove_targets(&uninstall_target_isobin_manifest);

        self.install_service
            .run_install(
                &workspace,
                &tmp_workspace,
                &save_isobin_manifest,
                &specified_isobin_manifest,
                &install_target_isobin_manifest,
                &uninstall_target_isobin_manifest,
                sync_service_option.quiet,
            )
            .await
    }
}

#[derive(Getters)]
pub struct SyncServiceOptionBase<P> {
    quiet: bool,
    force: bool,
    isobin_manifest_path: P,
}
pub type SyncServiceOption = SyncServiceOptionBase<Option<PathBuf>>;

pub type FixedSyncServiceOption = SyncServiceOptionBase<PathBuf>;

impl SyncServiceOption {
    async fn fix(self) -> Result<FixedSyncServiceOption> {
        let isobin_manifest_path =
            isobin_manifest_path_canonicalize(self.isobin_manifest_path).await?;
        Ok(FixedSyncServiceOption {
            quiet: self.quiet,
            force: self.force,
            isobin_manifest_path,
        })
    }
}

#[derive(Default)]
pub struct SyncServiceOptionBuilder {
    quiet: bool,
    force: bool,
    isobin_manifest_path: Option<PathBuf>,
}

impl SyncServiceOptionBuilder {
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
    pub fn build(self) -> SyncServiceOption {
        SyncServiceOption {
            quiet: self.quiet,
            force: self.force,
            isobin_manifest_path: self.isobin_manifest_path,
        }
    }
}
