use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use semver::Version;
use serde_derive::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    paths::workspace::Workspace,
    providers::ProviderKind,
    utils::file_modified::{has_file_diff_in_dir, FILE_MODIFIED_CACHE_MAP_FILE_NAME},
    IsobinConfigError, Result,
};

use super::home::CargoWorkspace;
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, new, Default, Getters)]
pub struct CargoConfig {
    #[serde(serialize_with = "toml::ser::tables_last")]
    installs: HashMap<String, CargoInstallDependency>,
}

impl CargoConfig {
    pub fn filter_target(&self, targets: &[String]) -> Self {
        let mut new_installs = HashMap::default();
        for target in targets.iter() {
            if let Some(install) = self.installs.get(target) {
                new_installs.insert(target.to_string(), install.clone());
            }
        }
        Self::new(new_installs)
    }

    pub fn merge(base_config: &Self, new_config: &Self) -> Self {
        let mut new_installs = base_config.installs().clone();
        for (name, install) in new_config.installs().iter() {
            new_installs.insert(name.to_string(), install.clone());
        }
        Self::new(new_installs)
    }

    pub async fn get_need_install_config(
        base: &Self,
        old: &Self,
        workspace: &Workspace,
    ) -> Result<Self> {
        let mut new_cargo_config = Self::default();
        let cargo_workspace = CargoWorkspace::from_workspace(workspace);
        for (name, dependency) in base.installs().iter() {
            if let Some(old_dependency) = old.installs().get(name) {
                if dependency != old_dependency
                    || Self::check_need_build_in_path(name, dependency, &cargo_workspace).await?
                {
                    new_cargo_config
                        .installs
                        .insert(name.to_string(), dependency.clone());
                }
            } else {
                new_cargo_config
                    .installs
                    .insert(name.to_string(), dependency.clone());
            }
        }
        Ok(new_cargo_config)
    }
    async fn check_need_build_in_path(
        name: &str,
        dependency: &CargoInstallDependency,
        cargo_workspace: &CargoWorkspace,
    ) -> Result<bool> {
        match dependency {
            CargoInstallDependency::Simple(_) => Ok(false),
            CargoInstallDependency::Detailed(dependency) => {
                if let Some(path) = dependency.path() {
                    let file_modified_cache_map_file_path = cargo_workspace
                        .cargo_home_dir()
                        .join(name)
                        .join(FILE_MODIFIED_CACHE_MAP_FILE_NAME);
                    let modified_cache_map_data =
                        fs::read(file_modified_cache_map_file_path).await?;
                    let modified_cache_map = serde_json::from_slice(&modified_cache_map_data)?;
                    has_file_diff_in_dir(
                        path,
                        vec!["rs".into()],
                        vec!["Cargo.toml".into(), "Cargo.lock".into()],
                        vec![],
                        modified_cache_map,
                    )
                    .await
                } else {
                    Ok(false)
                }
            }
        }
    }
    pub fn validate(&self) -> Result<()> {
        let errs = self
            .installs
            .iter()
            .map(|(name, install)| match install.validate() {
                Ok(_) => Ok(()),
                Err(err) => {
                    Err(
                        IsobinConfigError::new_validate(ProviderKind::Cargo, name.to_string(), err)
                            .into(),
                    )
                }
            })
            .filter(|r| r.is_err())
            .map(|r| r.unwrap_err())
            .collect::<Vec<_>>();
        if errs.is_empty() {
            Ok(())
        } else {
            Err(IsobinConfigError::MultiValidate(errs).into())
        }
    }
    pub fn fix(&mut self, isobin_config_dir: &Path) {
        for (_, install) in self.installs.iter_mut() {
            install.fix(isobin_config_dir)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum CargoInstallDependency {
    Simple(Version),
    Detailed(CargoInstallDependencyDetail),
}

impl CargoInstallDependency {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Simple(_) => Ok(()),
            Self::Detailed(dependency) => dependency.validate(),
        }
    }

    pub fn fix(&mut self, isobin_config_dir: &Path) {
        match self {
            Self::Simple(_) => {}
            Self::Detailed(dependency) => dependency.fix(isobin_config_dir),
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, PartialEq, Default, Serialize, new, Deserialize, Getters)]
pub struct CargoInstallDependencyDetail {
    bins: Option<Vec<String>>,
    version: Option<Version>,
    registry: Option<String>,
    index: Option<String>,
    path: Option<PathBuf>,
    git: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    rev: Option<String>,
    features: Option<Vec<String>>,
    #[serde(alias = "no-default-features", alias = "noDefaultFeatures")]
    no_default_features: Option<bool>,
    #[serde(alias = "all-features", alias = "allFeatures")]
    all_features: Option<bool>,
}

impl CargoInstallDependencyDetail {
    pub fn fix(&mut self, isobin_config_dir: &Path) {
        if let Some(path) = &self.path {
            self.path = Some(isobin_config_dir.join(path));
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.version().is_none() && self.path.is_none() && self.git.is_none() {
            Err(anyhow!(
                "cargo install dependency should have version or path or git."
            ))
        } else {
            Ok(())
        }
    }

    pub fn from_version(version: impl Into<Version>) -> Self {
        Self {
            version: Some(version.into()),
            ..Default::default()
        }
    }
}
