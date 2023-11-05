use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use semver::Version;
use serde_derive::{Deserialize, Serialize};

use crate::{providers::ProviderKind, IsobinConfigError, Result};
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, new, Default, Getters)]
pub struct CargoConfig {
    #[serde(serialize_with = "toml::ser::tables_last")]
    installs: HashMap<String, CargoInstallDependency>,
}

impl CargoConfig {
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
