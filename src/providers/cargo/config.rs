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
    pub fn to_detail(&self) -> CargoInstallDependencyDetail {
        match self {
            Self::Simple(version) => CargoInstallDependencyDetail {
                version: Some(version.into()),
                ..Default::default()
            },
            Self::Detailed(detail) => detail.clone(),
        }
    }
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
}
