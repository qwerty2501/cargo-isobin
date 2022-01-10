use super::*;
use crate::utils::{
    io_ext,
    serde_ext::{Json, SerdeExtError, Toml, Yaml},
};
use async_std::path::Path;

use providers::cargo::CargoConfig;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct IsobinConfig {
    #[serde(default)]
    cargo: CargoConfig,
}

#[derive(thiserror::Error, Debug, new)]
pub enum IsobinConfigError {
    #[error("{0}")]
    Serde(#[from] SerdeExtError),
    #[error("The target file does not have extension\npath:{path}")]
    NothingFileExtension { path: String },

    #[error("The target file has unknown extension\npath:{path}\nextension:{extension}")]
    UnknownFileExtension { path: String, extension: String },
}

type Result<T> = std::result::Result<T, IsobinConfigError>;

impl IsobinConfig {
    #[allow(dead_code)]
    pub async fn parse_from_file(path: impl AsRef<Path>) -> Result<IsobinConfig> {
        let file_extension = Self::get_file_extension(path.as_ref())?;
        Self::parse(file_extension, path).await
    }

    fn get_file_extension(path: impl AsRef<Path>) -> Result<ConfigFileExtensions> {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                IsobinConfigError::new_nothing_file_extension(io_ext::path_to_string(path.as_ref()))
            })?;

        const TOML_EXTENSION: &str = "toml";
        const YAML_EXTENSION: &str = "yaml";
        const YML_EXTENSION: &str = "yml";
        const JSON_EXTENSION: &str = "json";
        match extension {
            TOML_EXTENSION => Ok(ConfigFileExtensions::Toml),
            YML_EXTENSION | YAML_EXTENSION => Ok(ConfigFileExtensions::Yaml),
            JSON_EXTENSION => Ok(ConfigFileExtensions::Json),
            _ => Err(IsobinConfigError::new_unknown_file_extension(
                io_ext::path_to_string(path.as_ref()),
                extension.to_string(),
            )),
        }
    }

    async fn parse(
        file_extension: ConfigFileExtensions,
        path: impl AsRef<Path>,
    ) -> Result<IsobinConfig> {
        match file_extension {
            ConfigFileExtensions::Toml => Ok(Toml::parse_from_file(path).await?),
            ConfigFileExtensions::Yaml => Ok(Yaml::parse_from_file(path).await?),
            ConfigFileExtensions::Json => Ok(Json::parse_from_file(path).await?),
        }
    }
}

#[derive(PartialEq, Debug)]
enum ConfigFileExtensions {
    Yaml,
    Toml,
    Json,
}

#[cfg(test)]
mod tests {
    use crate::utils::io_ext;

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
        let actual = IsobinConfig::parse_from_file(dir.join(path)).await.unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(
        ConfigFileExtensions::Toml,
        "testdata/isobin_configs/default_load.toml",
        tool_config(cargo_install_dependencies())
    )]
    #[case(
        ConfigFileExtensions::Yaml,
        "testdata/isobin_configs/default_load.yaml",
        tool_config(cargo_install_dependencies())
    )]
    async fn isobin_config_from_str_works(
        #[case] ft: ConfigFileExtensions,
        #[case] path: impl AsRef<Path>,
        #[case] expected: IsobinConfig,
    ) {
        let path = current_source_dir!().join(path);
        let actual = IsobinConfig::parse(ft, path).await.unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    fn with_current_source_dir(path: &str) -> String {
        let r = current_source_dir!().join(path);
        io_ext::path_to_string(r)
    }

    #[rstest]
    #[case(
        ConfigFileExtensions::Toml,
        "testdata/isobin_configs/default_load.yaml",
        IsobinConfigError::new_serde(
            SerdeExtError::new_deserialize_with_hint(
                anyhow!("expected an equals, found a colon at line 1 column 6"),
                with_current_source_dir("testdata/isobin_configs/default_load.yaml"),
                    "cargo:\n_____^\n".into()),
            ),
        )]
    #[case(
        ConfigFileExtensions::Yaml,
        "testdata/isobin_configs/default_load.toml",
        IsobinConfigError::new_serde(
            SerdeExtError::new_deserialize_with_hint(
                anyhow!("did not find expected <document start> at line 2 column 1\n\nCaused by:\n    did not find expected <document start> at line 2 column 1"),
                with_current_source_dir("testdata/isobin_configs/default_load.toml"),
                    "[cargo.installs]\ncargo-make = \"2.0\"\ncomrak = \"1.0\"\n_^\n".into()),
            ),
        )]
    async fn isobin_config_from_str_error_works(
        #[case] ft: ConfigFileExtensions,
        #[case] path: impl AsRef<Path>,
        #[case] expected: IsobinConfigError,
    ) {
        let path = current_source_dir!().join(path);
        let result = IsobinConfig::parse(ft, path).await;
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
