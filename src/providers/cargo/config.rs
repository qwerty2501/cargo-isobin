use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug, PartialEq, new, Default, Getters)]
pub struct CargoConfig {
    #[serde(serialize_with = "toml::ser::tables_last")]
    installs: HashMap<String, CargoInstallDependency>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum CargoInstallDependency {
    Simple(String),
    Detailed(CargoInstallDependencyDetail),
}

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, PartialEq, Default, Serialize, new, Deserialize, Getters)]
pub struct CargoInstallDependencyDetail {
    bins: Option<Vec<String>>,
    version: Option<String>,
    registry: Option<String>,
    index: Option<String>,
    path: Option<String>,
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
    pub fn from_version(version: impl Into<String>) -> Self {
        Self {
            version: Some(version.into()),
            ..Default::default()
        }
    }
}
