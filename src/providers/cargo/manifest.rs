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
    utils::file_modified::{
        has_file_diff_in_dir, FileDiffHelpers, FILE_MODIFIED_CACHE_MAP_FILE_NAME,
    },
    IsobinManifestError, Result,
};

use super::home::CargoWorkspace;
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, new, Default, Getters)]
pub struct CargoManifest {
    #[serde(serialize_with = "toml::ser::tables_last")]
    dependencies: HashMap<String, CargoInstallDependency>,
}

impl CargoManifest {
    pub fn filter_target(&self, targets: &[String]) -> Self {
        let mut new_dependencies = HashMap::default();
        for target in targets.iter() {
            if let Some(dependency) = self.dependencies.get(target) {
                new_dependencies.insert(target.to_string(), dependency.clone());
            }
        }
        Self::new(new_dependencies)
    }

    pub fn merge(base_manifest: &Self, new_manifest: &Self) -> Self {
        let mut new_dependencies = base_manifest.dependencies().clone();
        for (name, dependency) in new_manifest.dependencies().iter() {
            new_dependencies.insert(name.to_string(), dependency.clone());
        }
        Self::new(new_dependencies)
    }
    pub fn remove_targets(base_manifest: &Self, remove_target_manifest: &Self) -> Self {
        let mut new_manifest = base_manifest.clone();
        for name in base_manifest.dependencies().keys() {
            if remove_target_manifest.dependencies().get(name).is_some() {
                new_manifest.dependencies.remove(name);
            }
        }
        new_manifest
    }

    pub async fn get_need_install_dependency_manifest(
        base: &Self,
        old: &Self,
        workspace: &Workspace,
    ) -> Result<Self> {
        let mut new_cargo_manifest = Self::default();
        let cargo_workspace = CargoWorkspace::from_workspace(workspace);
        for (name, dependency) in base.dependencies().iter() {
            if let Some(old_dependency) = old.dependencies().get(name) {
                if dependency != old_dependency
                    || Self::check_need_build_in_path(name, dependency, &cargo_workspace).await?
                {
                    new_cargo_manifest
                        .dependencies
                        .insert(name.to_string(), dependency.clone());
                }
            } else {
                new_cargo_manifest
                    .dependencies
                    .insert(name.to_string(), dependency.clone());
            }
        }
        Ok(new_cargo_manifest)
    }

    pub async fn get_need_uninstall_dependency_manifest(base: &Self, old: &Self) -> Result<Self> {
        let mut new_cargo_manifest = Self::default();
        for (name, dependency) in old.dependencies().iter() {
            if base.dependencies().get(name).is_none() {
                new_cargo_manifest
                    .dependencies
                    .insert(name.to_string(), dependency.clone());
            }
        }
        Ok(new_cargo_manifest)
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
                        FileDiffHelpers {
                            target_exts: vec!["rs".into()],
                            target_file_names: vec!["Cargo.toml".into(), "Cargo.lock".into()],
                            exclude_file_names: vec![],
                            exclude_dir_names: vec![".git".into(), "target".into()],
                        },
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
            .dependencies
            .iter()
            .map(|(name, dependency)| match dependency.validate() {
                Ok(_) => Ok(()),
                Err(err) => Err(IsobinManifestError::new_validate(
                    ProviderKind::Cargo,
                    name.to_string(),
                    err,
                )
                .into()),
            })
            .filter(|r| r.is_err())
            .map(|r| r.unwrap_err())
            .collect::<Vec<_>>();
        if errs.is_empty() {
            Ok(())
        } else {
            Err(IsobinManifestError::MultiValidate(errs).into())
        }
    }
    pub fn fix(mut self, isobin_manifest_dir: &Path) -> Self {
        for (name, dependency) in self.dependencies.clone().into_iter() {
            self.dependencies
                .insert(name, dependency.fix(isobin_manifest_dir));
        }
        self
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

    pub fn fix(self, isobin_manifest_dir: &Path) -> Self {
        match self {
            Self::Simple(_) => self,
            Self::Detailed(dependency) => Self::Detailed(dependency.fix(isobin_manifest_dir)),
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
    #[serde(skip)]
    absolute_path: Option<PathBuf>,
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
    pub fn fix(mut self, isobin_manifest_dir: &Path) -> Self {
        if let Some(path) = &self.path {
            self.absolute_path = Some(isobin_manifest_dir.join(path));
        }
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.version().is_none() && self.path.is_none() && self.git.is_none() {
            Err(anyhow!(
                "cargo dependency dependency should have version or path or git."
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
