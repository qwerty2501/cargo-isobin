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
impl CargoInstallDependency {
    pub fn into_detail(self) -> CargoInstallDependencyDetail {
        match self {
            Self::Simple(version) => CargoInstallDependencyDetail {
                version: Some(version),
                ..Default::default()
            },
            Self::Detailed(detail) => detail,
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters)]
pub struct CargoInstallDependencyDetail {
    #[serde(default)]
    pub bins: Vec<String>,
    pub version: Option<String>,
    pub registry: Option<String>,
    #[serde(alias = "registry-index", alias = "registryIndex")]
    pub registry_index: Option<String>,
    pub path: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(alias = "default-features", alias = "defaultFeatures")]
    pub default_features: Option<bool>,
}
