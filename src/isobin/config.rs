use super::*;
use std::io::Read;
use std::{fs::File, path::Path};

use providers::cargo::CargoInstallConfig;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct IsobinInstallConfig {
    #[serde(default)]
    install: InstallConfig,
}

impl IsobinInstallConfig {
    #[allow(dead_code)]
    pub fn from_path(path: impl AsRef<Path>) -> Result<IsobinInstallConfig> {
        let mut file = File::open(path.as_ref())
            .map_err(|e| Error::new_read_isobin_install_config_error(e.into()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| Error::new_read_isobin_install_config_error(e.into()))?;
        Self::from_toml_str(&content)
    }
    fn from_toml_str(s: &str) -> Result<IsobinInstallConfig> {
        let tool_config: IsobinInstallConfig = toml::from_str(s)
            .map_err(|e| Error::new_parse_isobin_install_config_error(e.into()))?;
        Ok(tool_config)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Default)]
pub struct InstallConfig {
    #[serde(default)]
    cargo: CargoInstallConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use providers::cargo::{CargoInstallDependency, CargoInstallDependencyDetail};

    #[fixture]
    fn cargo_install_dependencies() -> Vec<(String, CargoInstallDependency)> {
        [
            ("comrak", CargoInstallDependency::Simple("1.0".into())),
            ("cargo-make", CargoInstallDependency::Simple("2.0".into())),
        ]
        .into_iter()
        .map(|(name, v)| (name.to_string(), v))
        .collect()
    }

    #[fixture]
    fn tool_config(
        cargo_install_dependencies: Vec<(String, CargoInstallDependency)>,
    ) -> IsobinInstallConfig {
        IsobinInstallConfig {
            install: InstallConfig {
                cargo: CargoInstallConfig::new(cargo_install_dependencies.into_iter().collect()),
            },
        }
    }

    #[fixture]
    #[allow(clippy::field_reassign_with_default)]
    fn table_cargos() -> Vec<(String, CargoInstallDependency)> {
        let mut cargos = vec![];
        let mut comrak_dependency_detail = CargoInstallDependencyDetail::default();
        comrak_dependency_detail.version = Some("1.0".into());
        comrak_dependency_detail.git = Some("git@github.com:kivikakk/comrak.git".into());
        cargos.push((
            "comrak".to_string(),
            CargoInstallDependency::Detailed(comrak_dependency_detail),
        ));

        let mut cargo_make_dependency_detail = CargoInstallDependencyDetail::default();
        cargo_make_dependency_detail.version = Some("2.0".into());
        cargos.push((
            "cargo-make".to_string(),
            CargoInstallDependency::Detailed(cargo_make_dependency_detail),
        ));
        cargos
    }

    #[fixture]
    fn empty_cargos() -> Vec<(String, CargoInstallDependency)> {
        return vec![];
    }

    #[rstest]
    #[case(tool_config(cargo_install_dependencies()),include_str!("testdata/tool_config_from_str_works/default_load.toml"))]
    #[case(tool_config(table_cargos()),include_str!("testdata/tool_config_from_str_works/description_load.toml"))]
    #[case(tool_config(empty_cargos()),include_str!("testdata/tool_config_from_str_works/empty.toml"))]
    #[case(tool_config(empty_cargos()),include_str!("testdata/tool_config_from_str_works/empty_cargo.toml"))]
    fn tool_config_from_str_works(
        #[case] expected: IsobinInstallConfig,
        #[case] config_toml_str: &str,
    ) {
        let result = IsobinInstallConfig::from_toml_str(config_toml_str);
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
