use super::*;
use async_std::io::ReadExt;
use async_std::{
    fs::File,
    path::{Path, PathBuf},
};

use providers::cargo::CargoConfig;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct IsobinConfig {
    #[serde(default)]
    cargo: CargoConfig,
}

#[derive(thiserror::Error, Debug, new)]
pub enum IsobinConfigError {
    #[error("Failed read isobin install config\npath:{path:?}\nerror:{source}")]
    ReadIsobinConfig {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    #[error("Failed parse isobin config\npath:{path:?}\nerror:{source}")]
    ParseIsobinConfig {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    #[error("The target file does not have extension\npath:{path:?}")]
    NothingFileExtension { path: PathBuf },

    #[error("The target file has unknown extension\npath:{path:?}\nextension:{extension}")]
    UnknownFileExtension { path: PathBuf, extension: String },
}

type Result<T> = std::result::Result<T, IsobinConfigError>;

impl IsobinConfig {
    #[allow(dead_code)]
    pub async fn parse_from_path(path: impl AsRef<Path>) -> Result<IsobinConfig> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .await
            .map_err(|e| IsobinConfigError::new_read_isobin_config(path.into(), e.into()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .await
            .map_err(|e| IsobinConfigError::new_read_isobin_config(path.into(), e.into()))?;
        let file_extension = Self::get_file_extension(path)?;
        Self::parse(&content, file_extension, path)
    }

    fn get_file_extension(path: impl AsRef<Path>) -> Result<ConfigFileExtensions> {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| IsobinConfigError::new_nothing_file_extension(path.as_ref().into()))?;

        const TOML_EXTENSION: &str = "toml";
        const YAML_EXTENSION: &str = "yaml";
        const YML_EXTENSION: &str = "yml";
        match extension {
            TOML_EXTENSION => Ok(ConfigFileExtensions::Toml),
            YML_EXTENSION | YAML_EXTENSION => Ok(ConfigFileExtensions::Yaml),
            _ => Err(IsobinConfigError::new_unknown_file_extension(
                path.as_ref().into(),
                extension.to_string(),
            )),
        }
    }

    fn parse(
        s: &str,
        file_extension: ConfigFileExtensions,
        path: impl AsRef<Path>,
    ) -> Result<IsobinConfig> {
        match file_extension {
            ConfigFileExtensions::Toml => Self::parse_toml(s, path),
            ConfigFileExtensions::Yaml => Self::parse_yaml(s, path),
        }
    }

    fn parse_toml(s: &str, path: impl AsRef<Path>) -> Result<IsobinConfig> {
        let isobin_config: IsobinConfig = toml::from_str(s).map_err(|e| {
            IsobinConfigError::new_parse_isobin_config(path.as_ref().into(), e.into())
        })?;
        Ok(isobin_config)
    }
    fn parse_yaml(s: &str, path: impl AsRef<Path>) -> Result<IsobinConfig> {
        let isobin_config: IsobinConfig = serde_yaml::from_str(s).map_err(|e| {
            IsobinConfigError::new_parse_isobin_config(path.as_ref().into(), e.into())
        })?;
        Ok(isobin_config)
    }
}

#[derive(PartialEq, Debug)]
enum ConfigFileExtensions {
    Yaml,
    Toml,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use providers::cargo::{CargoInstallDependency, CargoInstallDependencyDetail};

    #[rstest]
    #[case(
        "testdata/isobin_configs/default_load.toml",
        tool_config(cargo_install_dependencies())
    )]
    async fn isobin_config_from_path_works(#[case] path: &str, #[case] expected: IsobinConfig) {
        let dir = current_source_dir!();
        let actual = IsobinConfig::parse_from_path(dir.join(path)).await.unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(include_str!("testdata/isobin_configs/default_load.toml"),ConfigFileExtensions::Toml,"foo.toml",tool_config(cargo_install_dependencies()))]
    #[case(include_str!("testdata/isobin_configs/default_load.yaml"),ConfigFileExtensions::Yaml,"foo.yaml",tool_config(cargo_install_dependencies()))]
    fn isobin_config_from_str_works(
        #[case] config_str: &str,
        #[case] ft: ConfigFileExtensions,
        #[case] path: &str,
        #[case] expected: IsobinConfig,
    ) {
        let actual = IsobinConfig::parse(config_str, ft, path).unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(include_str!("testdata/isobin_configs/default_load.yaml"),ConfigFileExtensions::Toml,"foo.toml",IsobinConfigError::new_parse_isobin_config("foo.toml".into(),anyhow!("expected an equals, found a colon at line 1 column 6")))]
    #[case(include_str!("testdata/isobin_configs/default_load.toml"),ConfigFileExtensions::Yaml,"foo.yaml",IsobinConfigError::new_parse_isobin_config("foo.yaml".into(),anyhow!("did not find expected <document start> at line 2 column 1\n\nCaused by:\n    did not find expected <document start> at line 2 column 1")))]
    fn isobin_config_from_str_error_works(
        #[case] config_str: &str,
        #[case] ft: ConfigFileExtensions,
        #[case] path: &str,
        #[case] expected: IsobinConfigError,
    ) {
        let result = IsobinConfig::parse(config_str, ft, path);
        assert_error_result!(expected, result);
    }

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
    ) -> IsobinConfig {
        IsobinConfig {
            cargo: CargoConfig::new(cargo_install_dependencies.into_iter().collect()),
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
    #[case(include_str!("testdata/isobin_configs/default_load.toml"),tool_config(cargo_install_dependencies()))]
    #[case(include_str!("testdata/isobin_configs/description_load.toml"),tool_config(table_cargos()))]
    #[case(include_str!("testdata/isobin_configs/empty.toml"),tool_config(empty_cargos()))]
    #[case(include_str!("testdata/isobin_configs/empty_cargo.toml"),tool_config(empty_cargos()))]
    fn isobin_config_from_toml_str_works(
        #[case] config_toml_str: &str,
        #[values("foo.toml")] path: &str,
        #[case] expected: IsobinConfig,
    ) {
        let actual = IsobinConfig::parse_toml(config_toml_str, path).unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(include_str!("testdata/isobin_configs/default_load.yaml"),tool_config(cargo_install_dependencies()))]
    #[case(include_str!("testdata/isobin_configs/description_load.yaml"),tool_config(table_cargos()))]
    fn isobin_config_from_yaml_str_works(
        #[case] config_toml_str: &str,
        #[values("foo.yaml")] path: &str,
        #[case] expected: IsobinConfig,
    ) {
        let actual = IsobinConfig::parse_yaml(config_toml_str, path).unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(include_str!("testdata/isobin_configs/empty.yaml"),IsobinConfigError::new_parse_isobin_config("foo.yaml".into(), anyhow!("EOF while parsing a value")))]
    #[case(include_str!("testdata/isobin_configs/empty_cargo.yaml"),IsobinConfigError::new_parse_isobin_config("foo.yaml".into(),anyhow!("cargo.installs: invalid type: unit value, expected a map at line 3 column 1")))]
    fn isobin_config_from_yaml_str_error_works(
        #[case] config_toml_str: &str,
        #[values("foo.yaml")] path: &str,
        #[case] expected: IsobinConfigError,
    ) {
        let result = IsobinConfig::parse_yaml(config_toml_str, path);
        assert_error_result!(expected, result);
    }

    #[rstest]
    #[case("foo.yaml", ConfigFileExtensions::Yaml)]
    #[case("foo.yml", ConfigFileExtensions::Yaml)]
    #[case("foo.toml", ConfigFileExtensions::Toml)]
    fn get_config_file_extension_works(#[case] path: &str, #[case] expected: ConfigFileExtensions) {
        let actual = IsobinConfig::get_file_extension(path).unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("foo.fm", IsobinConfigError::new_unknown_file_extension("foo.fm".into(), "fm".into()))]
    #[case("foo", IsobinConfigError::new_nothing_file_extension("foo".into()))]
    fn get_config_file_extension_error_works(
        #[case] path: &str,
        #[case] expected: IsobinConfigError,
    ) {
        let result = IsobinConfig::get_file_extension(path);
        assert_error_result!(expected, result);
    }
}
