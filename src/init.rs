use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use anyhow::bail;

use crate::{
    paths::isobin_manifest::{isobin_manifest_dir, make_isobin_manifest_paths, BASE_FILE_NAME},
    utils::fs_ext::smart_create_file,
    Result,
};

#[derive(Default)]
pub struct InitService {}

impl InitService {
    pub async fn init(&self, init_service_option: InitServiceOption) -> Result<()> {
        let isobin_manifest_path =
            if let Some(isobin_manifest_path) = init_service_option.isobin_manifest_path() {
                isobin_manifest_path.clone()
            } else {
                current_dir()?.join(BASE_FILE_NAME.to_string() + ".yaml")
            };

        let isobin_manifest_dir = isobin_manifest_dir(&isobin_manifest_path)?;
        Self::check_isobin_path_exists(&isobin_manifest_path, isobin_manifest_dir)?;

        let check_isobin_paths = make_isobin_manifest_paths(isobin_manifest_dir);
        for check_isobin_path in check_isobin_paths.iter() {
            if check_isobin_path.exists() {
                Self::check_isobin_path_exists(check_isobin_path, isobin_manifest_dir)?;
            }
        }

        let _ = smart_create_file(isobin_manifest_path).await;
        Ok(())
    }
    fn check_isobin_path_exists(
        isobin_manifest_path: impl AsRef<Path>,
        isobin_manifest_dir: impl AsRef<Path>,
    ) -> Result<()> {
        if isobin_manifest_path.as_ref().exists() {
            bail!(
                "The isobin manifest already exists in {}",
                isobin_manifest_dir.as_ref().display()
            );
        }
        Ok(())
    }
}

#[derive(Getters)]
pub struct InitServiceOptionBase<P> {
    quiet: bool,
    isobin_manifest_path: P,
}

pub type InitServiceOption = InitServiceOptionBase<Option<PathBuf>>;

#[derive(Default)]
pub struct InitServiceOptionBuilder {
    quiet: bool,
    isobin_manifest_path: Option<PathBuf>,
}

impl InitServiceOptionBuilder {
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn isobin_manifest_path(mut self, isobin_manifest_path: PathBuf) -> Self {
        self.isobin_manifest_path = Some(isobin_manifest_path);
        self
    }

    pub fn build(self) -> InitServiceOption {
        InitServiceOption {
            quiet: self.quiet,
            isobin_manifest_path: self.isobin_manifest_path,
        }
    }
}
