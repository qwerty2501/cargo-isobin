use std::collections::HashMap;

pub use cargo_toml::Dependency as CargoInstallDependency;
pub use cargo_toml::DependencyDetail as CargoInstallDependencyDetail;
use serde_derive::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug, PartialEq, new, Default)]
pub struct CargoConfig {
    #[serde(serialize_with = "toml::ser::tables_last")]
    installs: HashMap<String, CargoInstallDependency>,
}
