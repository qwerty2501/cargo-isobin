use super::*;
use crate::{
    paths::{isobin_config::IsobinConfigPathError, workspace::Workspace},
    providers::ProviderKind,
    utils::{
        fs_ext, io_ext,
        serde_ext::{Json, Toml, Yaml},
    },
};
use std::path::{Path, PathBuf};

use providers::cargo::CargoConfig;
use serde_derive::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt};

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Getters, Default, new)]
pub struct IsobinConfig {
    #[serde(default)]
    cargo: CargoConfig,
}

#[derive(thiserror::Error, Debug, new)]
pub enum IsobinConfigError {
    #[error("The target file does not have extension\npath:{path}")]
    NothingFileExtension { path: String },

    #[error("The target file has unknown extension\npath:{path}\nextension:{extension}")]
    UnknownFileExtension { path: String, extension: String },

    #[error("{provider}/{name}\n{error}")]
    Validate {
        provider: ProviderKind,
        name: String,
        error: Error,
    },
    #[error("{0:#?}")]
    MultiValidate(Vec<Error>),
}

impl IsobinConfig {
    pub async fn load_from_file(path: impl AsRef<Path>) -> Result<IsobinConfig> {
        let file_extension = Self::get_file_extension(path.as_ref())?;
        let mut isobin_config = Self::parse(file_extension, path.as_ref()).await?;
        let isobin_config_dir = path
            .as_ref()
            .parent()
            .ok_or_else(IsobinConfigPathError::new_not_found_isobin_config)?;

        isobin_config.fix(isobin_config_dir);
        isobin_config.validate()?;
        Ok(isobin_config)
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
            )
            .into()),
        }
    }
    pub fn validate(&self) -> Result<()> {
        self.cargo.validate()
    }
    pub fn fix(&mut self, isobin_config_dir: &Path) {
        self.cargo.fix(isobin_config_dir)
    }

    pub fn filter_target(&self, targets: &[String]) -> Self {
        Self::new(self.cargo().filter_target(targets))
    }

    pub fn merge(base_config: &Self, new_config: &Self) -> Self {
        Self::new(CargoConfig::merge(base_config.cargo(), new_config.cargo()))
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
    pub async fn get_need_install_config(
        base: &Self,
        old: &Self,
        workspace: &Workspace,
    ) -> Result<Self> {
        Ok(Self {
            cargo: CargoConfig::get_need_install_config(base.cargo(), old.cargo(), workspace)
                .await?,
        })
    }
}

#[derive(PartialEq, Debug)]
enum ConfigFileExtensions {
    Yaml,
    Toml,
    Json,
}

pub struct IsobinConfigCache;

impl IsobinConfigCache {
    const ISOBIN_CONFIG_FILE_CACHE_NAME: &str = "isobin_cache.v1.json";
    fn make_cache_path(dir: impl AsRef<Path>) -> PathBuf {
        dir.as_ref().join(Self::ISOBIN_CONFIG_FILE_CACHE_NAME)
    }

    pub async fn lenient_load_cache_from_dir(dir: impl AsRef<Path>) -> IsobinConfig {
        let cache_file_path = Self::make_cache_path(dir);
        if cache_file_path.exists() {
            match Self::load_cache_from_path(cache_file_path).await {
                Ok(cache) => cache,
                Err(_) => IsobinConfig::default(),
            }
        } else {
            IsobinConfig::default()
        }
    }

    pub async fn save_cache_to_dir(
        isobin_config: &IsobinConfig,
        dir: impl AsRef<Path>,
    ) -> Result<()> {
        let cache_file_path = Self::make_cache_path(dir);
        let mut isobin_config_file_cache =
            fs_ext::open_file_create_if_not_exists(cache_file_path).await?;
        let sirialized_isobin_config = serde_json::to_vec(isobin_config)?;
        isobin_config_file_cache
            .write_all(&sirialized_isobin_config)
            .await?;
        Ok(())
    }
    async fn load_cache_from_path(cache_file_path: impl AsRef<Path>) -> Result<IsobinConfig> {
        let data = fs::read(cache_file_path).await?;
        Ok(serde_json::from_slice(&data)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::io_ext;

    use super::*;
    use anyhow::anyhow;
    use providers::cargo::{CargoInstallDependency, CargoInstallDependencyDetail};

    use semver::Version;
    use utils::serde_ext::{ErrorHint, SerdeExtError};

    #[rstest]
    #[case(
        "testdata/isobin_configs/default_load.toml",
        tool_config(cargo_install_dependencies())
    )]
    #[tokio::test]
    async fn isobin_config_from_path_works(#[case] path: &str, #[case] expected: IsobinConfig) {
        let dir = current_source_dir!();
        let actual = IsobinConfig::load_from_file(dir.join(path)).await.unwrap();
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
    #[tokio::test]
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
            SerdeExtError::new_deserialize_with_hint(
                anyhow!("expected an equals, found a colon at line 1 column 6"),
                with_current_source_dir("testdata/isobin_configs/default_load.yaml"),
                ErrorHint::new(1,6,include_str!("testdata/isobin_configs/default_load.yaml").into()),
            ),
        )]
    #[case(
        ConfigFileExtensions::Yaml,
        "testdata/isobin_configs/default_load.toml",
            SerdeExtError::new_deserialize_with_hint(
                anyhow!("did not find expected <document start> at line 2 column 1"),
                with_current_source_dir("testdata/isobin_configs/default_load.toml"),
                ErrorHint::new(2,1,include_str!("testdata/isobin_configs/default_load.toml").into()),
            ),
        )]
    #[case(
        ConfigFileExtensions::Json,
        "testdata/isobin_configs/default_load.toml",
            SerdeExtError::new_deserialize_with_hint(
                anyhow!("expected value at line 1 column 2"),
                with_current_source_dir("testdata/isobin_configs/default_load.toml"),
                ErrorHint::new(1,2,include_str!("testdata/isobin_configs/default_load.toml").into()),
            ),
        )]
    #[tokio::test]
    async fn isobin_config_from_str_error_works(
        #[case] ft: ConfigFileExtensions,
        #[case] path: impl AsRef<Path>,
        #[case] expected: SerdeExtError,
    ) {
        let path = current_source_dir!().join(path);
        let result = IsobinConfig::parse(ft, path).await;
        assert_error_result!(expected, result);
    }

    #[fixture]
    fn cargo_install_dependencies() -> Vec<(String, CargoInstallDependency)> {
        [
            (
                "comrak",
                CargoInstallDependency::Simple(Version::parse("1.0.0").unwrap()),
            ),
            (
                "cargo-make",
                CargoInstallDependency::Simple(Version::parse("2.0.0").unwrap()),
            ),
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
        let comrak_dependency_detail = CargoInstallDependencyDetail::new(
            Default::default(),
            Some(Version::parse("1.0.0").unwrap()),
            Default::default(),
            Default::default(),
            Default::default(),
            Some("git@github.com:kivikakk/comrak.git".into()),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );
        cargos.push((
            "comrak".to_string(),
            CargoInstallDependency::Detailed(comrak_dependency_detail),
        ));

        let cargo_make_dependency_detail = CargoInstallDependencyDetail::new(
            Default::default(),
            Some(Version::parse("2.0.0").unwrap()),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );
        cargos.push((
            "cargo-make".to_string(),
            CargoInstallDependency::Detailed(cargo_make_dependency_detail),
        ));
        cargos
    }

    #[fixture]
    fn empty_cargos() -> Vec<(String, CargoInstallDependency)> {
        vec![]
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
