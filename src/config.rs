use std::io::Read;
use std::{fs::File, path::Path};

use super::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ToolConfig {
    #[serde(rename = "install-dependencies")]
    install_dependencies: InstallDependencies,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct InstallDependencies {
    cargo: toml::value::Table,
}

impl ToolConfig {
    #[allow(dead_code)]
    pub fn from_path(path: impl AsRef<Path>) -> Result<ToolConfig> {
        let mut file = File::open(path.as_ref())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Self::from_str(&content)
    }
    fn from_str(s: &str) -> Result<ToolConfig> {
        let tool_config: ToolConfig = toml::from_str(s)?;
        Ok(tool_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[fixture]
    fn cargos() -> Vec<(String, toml::Value)> {
        [
            ("comrak", toml::Value::String("1.0".into())),
            ("cargo-make", toml::Value::String("2.0".into())),
        ]
        .into_iter()
        .map(|(name, v)| (name.to_string(), v))
        .collect()
    }

    #[fixture]
    fn tool_config(cargos: Vec<(String, toml::Value)>) -> ToolConfig {
        ToolConfig {
            install_dependencies: InstallDependencies {
                cargo: cargos.into_iter().collect(),
            },
        }
    }

    #[fixture]
    fn table_cargos() -> Vec<(String, toml::Value)> {
        [
            (
                "comrak",
                toml::Value::Table(
                    [
                        ("version", toml::Value::String("1.0".into())),
                        (
                            "git",
                            toml::Value::String("git@github.com:kivikakk/comrak.git".into()),
                        ),
                    ]
                    .into_iter()
                    .map(|(name, v)| (name.to_string(), v))
                    .collect(),
                ),
            ),
            (
                "cargo-make",
                toml::Value::Table(
                    [("version", toml::Value::String("2.0".into()))]
                        .into_iter()
                        .map(|(name, v)| (name.to_string(), v))
                        .collect(),
                ),
            ),
        ]
        .into_iter()
        .map(|(name, v)| (name.to_string(), v))
        .collect()
    }

    #[rstest]
    #[case(tool_config(cargos()),include_str!("testdata/tool_config_from_str_works/default_load.toml"))]
    #[case(tool_config(table_cargos()),include_str!("testdata/tool_config_from_str_works/description_load.toml"))]
    fn tool_config_from_str_works(#[case] expected: ToolConfig, #[case] config_toml_str: &str) {
        let result = ToolConfig::from_str(config_toml_str);
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
