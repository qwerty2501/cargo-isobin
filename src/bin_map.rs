use std::{collections::HashMap, path::Path};

use serde_derive::{Deserialize, Serialize};

use crate::{providers::ProviderKind, utils::serde_ext::Json, Result};

#[derive(Default, new, Getters, Deserialize, Serialize)]
pub struct BinMap {
    bin_dependencies: HashMap<String, BinDependency>,
}

impl BinMap {
    const BIN_MAP_FILE_NAME: &'static str = "bin_map.v1.json";
    pub fn insert(
        &mut self,
        file_name: String,
        bin_dependency: BinDependency,
    ) -> Option<BinDependency> {
        self.bin_dependencies.insert(file_name, bin_dependency)
    }

    pub fn remove(&mut self, file_name: &str) -> Option<BinDependency> {
        self.bin_dependencies.remove(file_name)
    }

    pub async fn lenient_load_from_dir(dir: impl AsRef<Path>) -> Result<BinMap> {
        let bin_map_path = dir.as_ref().join(Self::BIN_MAP_FILE_NAME);
        match Json::parse_from_file(bin_map_path).await {
            Ok(bin_map) => Ok(bin_map),
            Err(_) => Ok(BinMap::default()),
        }
    }

    pub async fn save_to_dir(bin_map: &BinMap, dir: impl AsRef<Path>) -> Result<()> {
        let bin_map_path = dir.as_ref().join(Self::BIN_MAP_FILE_NAME);
        Json::save_to_file(bin_map, bin_map_path).await
    }
}

#[derive(new, Getters, Deserialize, Serialize, PartialEq, Clone)]
pub struct BinDependency {
    provider_kind: ProviderKind,
    name: String,
    bin_file_name: String,
}
