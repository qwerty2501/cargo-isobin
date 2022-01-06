use std::collections::HashMap;
use std::io::Read;
use std::{fs::File, path::Path};

use super::*;
use serde_derive::{Deserialize, Serialize};

pub use cargo_toml::Dependency as InstallDependency;
pub use cargo_toml::DependencyDetail as InstallDependencyDetail;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct InstallConfig {
    #[serde(rename = "install-dependencies")]
    install_dependencies: HashMap<String, InstallDependency>,
}

impl InstallConfig {
    #[allow(dead_code)]
    pub fn from_path(path: impl AsRef<Path>) -> Result<InstallConfig> {
        let mut file = File::open(path.as_ref())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Self::from_str(&content)
    }
    fn from_str(s: &str) -> Result<InstallConfig> {
        let tool_config: InstallConfig = toml::from_str(s)?;
        Ok(tool_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[fixture]
    fn install_dependencies() -> Vec<(String, InstallDependency)> {
        [
            ("comrak", InstallDependency::Simple("1.0".into())),
            ("cargo-make", InstallDependency::Simple("2.0".into())),
        ]
        .into_iter()
        .map(|(name, v)| (name.to_string(), v))
        .collect()
    }

    #[fixture]
    fn tool_config(install_dependencies: Vec<(String, InstallDependency)>) -> InstallConfig {
        InstallConfig {
            install_dependencies: install_dependencies.into_iter().collect(),
        }
    }

    #[fixture]
    fn table_cargos() -> Vec<(String, InstallDependency)> {
        [
            (
                "comrak",
                InstallDependency::Detailed(InstallDependencyDetail {
                    version: Some("1.0".into()),
                    registry: None,
                    registry_index: None,
                    path: None,
                    git: Some("git@github.com:kivikakk/comrak.git".into()),
                    branch: None,
                    tag: None,
                    rev: None,
                    features: vec![],
                    optional: false,
                    default_features: None,
                    package: None,
                }),
            ),
            (
                "cargo-make",
                InstallDependency::Detailed(InstallDependencyDetail {
                    version: Some("2.0".into()),
                    registry: None,
                    registry_index: None,
                    path: None,
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    features: vec![],
                    optional: false,
                    default_features: None,
                    package: None,
                }),
            ),
        ]
        .into_iter()
        .map(|(name, v)| (name.to_string(), v))
        .collect()
    }

    #[rstest]
    #[case(tool_config(install_dependencies()),include_str!("testdata/tool_config_from_str_works/default_load.toml"))]
    #[case(tool_config(table_cargos()),include_str!("testdata/tool_config_from_str_works/description_load.toml"))]
    fn tool_config_from_str_works(#[case] expected: InstallConfig, #[case] config_toml_str: &str) {
        let result = InstallConfig::from_str(config_toml_str);
        match result {
            Ok(actual) => {
                pretty_assertions::assert_eq!(expected, actual);
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }
}
