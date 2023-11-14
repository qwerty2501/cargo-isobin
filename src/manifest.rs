use super::*;
use crate::{
    paths::{isobin_manifest::IsobinManifestPathError, workspace::Workspace},
    providers::ProviderKind,
    utils::{
        io_ext,
        serde_ext::{Json, Toml, Yaml},
    },
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use providers::cargo::CargoManifest;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Getters, Default, new)]
pub struct IsobinManifest {
    #[serde(default)]
    cargo: CargoManifest,
}

#[derive(thiserror::Error, Debug, new)]
pub enum IsobinManifestError {
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

impl IsobinManifest {
    pub async fn load_from_file(path: impl AsRef<Path>) -> Result<IsobinManifest> {
        let file_extension = Self::get_file_extension(path.as_ref())?;
        let isobin_manifest = Self::parse(file_extension, path.as_ref()).await?;
        let isobin_manifest_dir = path
            .as_ref()
            .parent()
            .ok_or_else(IsobinManifestPathError::new_not_found_isobin_manifest)?;

        let isobin_manifest = isobin_manifest.fix(isobin_manifest_dir);
        isobin_manifest.validate()?;
        Ok(isobin_manifest)
    }

    pub fn is_empty(&self) -> bool {
        self.cargo().dependencies().is_empty()
    }

    fn get_file_extension(path: impl AsRef<Path>) -> Result<ManifestFileExtensions> {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                IsobinManifestError::new_nothing_file_extension(io_ext::path_to_string(
                    path.as_ref(),
                ))
            })?;

        const TOML_EXTENSION: &str = "toml";
        const YAML_EXTENSION: &str = "yaml";
        const YML_EXTENSION: &str = "yml";
        const JSON_EXTENSION: &str = "json";
        match extension {
            TOML_EXTENSION => Ok(ManifestFileExtensions::Toml),
            YML_EXTENSION | YAML_EXTENSION => Ok(ManifestFileExtensions::Yaml),
            JSON_EXTENSION => Ok(ManifestFileExtensions::Json),
            _ => Err(IsobinManifestError::new_unknown_file_extension(
                io_ext::path_to_string(path.as_ref()),
                extension.to_string(),
            )
            .into()),
        }
    }
    pub fn validate(&self) -> Result<()> {
        self.cargo.validate()
    }
    pub fn fix(mut self, isobin_manifest_dir: &Path) -> Self {
        self.cargo = self.cargo.fix(isobin_manifest_dir);
        self
    }

    pub fn filter_target(&self, targets: &[String]) -> Self {
        Self::new(self.cargo().filter_target(targets))
    }

    pub fn merge(&self, new_manifest: &Self) -> Self {
        Self::new(self.cargo().merge(new_manifest.cargo()))
    }
    pub fn remove_targets(&self, remove_target_manifest: &Self) -> Self {
        Self::new(self.cargo().remove_targets(remove_target_manifest.cargo()))
    }

    async fn parse(
        file_extension: ManifestFileExtensions,
        path: impl AsRef<Path>,
    ) -> Result<IsobinManifest> {
        match file_extension {
            ManifestFileExtensions::Toml => Ok(Toml::parse_from_file(path).await?),
            ManifestFileExtensions::Yaml => Ok(Yaml::parse_from_file(path).await?),
            ManifestFileExtensions::Json => Ok(Json::parse_from_file(path).await?),
        }
    }
    pub async fn get_need_install_dependency_manifest(
        base: &Self,
        old: &Self,
        workspace: &Workspace,
    ) -> Result<Self> {
        Ok(Self {
            cargo: CargoManifest::get_need_install_dependency_manifest(
                base.cargo(),
                old.cargo(),
                workspace,
            )
            .await?,
        })
    }

    pub async fn get_need_uninstall_dependency_manifest(base: &Self, old: &Self) -> Result<Self> {
        Ok(Self {
            cargo: CargoManifest::get_need_uninstall_dependency_manifest(base.cargo(), old.cargo())
                .await?,
        })
    }
}

pub trait Manifest: Clone {
    type Dependency: Clone;
    fn dependencies(&self) -> &HashMap<String, Self::Dependency>;

    fn make_from_new_dependencies(&self, dependencies: HashMap<String, Self::Dependency>) -> Self;

    fn filter_target(&self, targets: &[String]) -> Self {
        let mut new_dependencies = HashMap::<String, Self::Dependency>::new();
        for target in targets.iter() {
            if let Some(dependency) = self.dependencies().get(target) {
                new_dependencies.insert(target.to_owned(), dependency.clone());
            }
        }
        self.make_from_new_dependencies(new_dependencies)
    }

    fn merge(&self, new_manifest: &Self) -> Self {
        let mut new_dependencies = self.dependencies().clone();
        for (name, dependency) in new_manifest.dependencies().iter() {
            new_dependencies.insert(name.to_string(), dependency.clone());
        }
        self.make_from_new_dependencies(new_dependencies)
    }

    fn remove_targets(&self, remove_target_manifest: &Self) -> Self {
        let mut new_dependencies = self.dependencies().clone();
        for name in self.dependencies().keys() {
            if remove_target_manifest.dependencies().get(name).is_some() {
                new_dependencies.remove(name);
            }
        }
        self.make_from_new_dependencies(new_dependencies)
    }
}

#[derive(PartialEq, Debug)]
enum ManifestFileExtensions {
    Yaml,
    Toml,
    Json,
}

pub struct IsobinManifestCache;

impl IsobinManifestCache {
    const ISOBIN_CONFIG_FILE_CACHE_NAME: &str = "isobin_cache.v1.json";
    fn make_cache_path(dir: impl AsRef<Path>) -> PathBuf {
        dir.as_ref().join(Self::ISOBIN_CONFIG_FILE_CACHE_NAME)
    }

    pub async fn lenient_load_cache_from_dir(dir: impl AsRef<Path>) -> IsobinManifest {
        let cache_file_path = Self::make_cache_path(dir);

        if cache_file_path.exists() {
            match Json::parse_or_default_if_not_found(cache_file_path).await {
                Ok(cache) => cache,
                Err(_) => IsobinManifest::default(),
            }
        } else {
            IsobinManifest::default()
        }
    }

    pub async fn save_cache_to_dir(
        isobin_manifest: &IsobinManifest,
        dir: impl AsRef<Path>,
    ) -> Result<()> {
        let cache_file_path = Self::make_cache_path(dir);
        Json::save_to_file(isobin_manifest, cache_file_path).await
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
        "testdata/isobin_manifests/default_load.toml",
        tool_manifest(cargo_install_dependencies())
    )]
    #[tokio::test]
    async fn isobin_manifest_from_path_works(#[case] path: &str, #[case] expected: IsobinManifest) {
        let dir = current_source_dir!();
        let actual = IsobinManifest::load_from_file(dir.join(path))
            .await
            .unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(
        ManifestFileExtensions::Toml,
        "testdata/isobin_manifests/default_load.toml",
        tool_manifest(cargo_install_dependencies())
    )]
    #[case(
        ManifestFileExtensions::Yaml,
        "testdata/isobin_manifests/default_load.yaml",
        tool_manifest(cargo_install_dependencies())
    )]
    #[tokio::test]
    async fn isobin_manifest_from_str_works(
        #[case] ft: ManifestFileExtensions,
        #[case] path: impl AsRef<Path>,
        #[case] expected: IsobinManifest,
    ) {
        let path = current_source_dir!().join(path);
        let actual = IsobinManifest::parse(ft, path).await.unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    fn with_current_source_dir(path: &str) -> String {
        let r = current_source_dir!().join(path);
        io_ext::path_to_string(r)
    }

    #[rstest]
    #[case(
        ManifestFileExtensions::Toml,
        "testdata/isobin_manifests/default_load.yaml",
            SerdeExtError::new_deserialize_with_hint(
                anyhow!("expected an equals, found a colon at line 1 column 6"),
                with_current_source_dir("testdata/isobin_manifests/default_load.yaml"),
                ErrorHint::new(1,6,include_str!("testdata/isobin_manifests/default_load.yaml").into()),
            ),
        )]
    #[case(
        ManifestFileExtensions::Yaml,
        "testdata/isobin_manifests/default_load.toml",
            SerdeExtError::new_deserialize_with_hint(
                anyhow!("did not find expected <document start> at line 2 column 1"),
                with_current_source_dir("testdata/isobin_manifests/default_load.toml"),
                ErrorHint::new(2,1,include_str!("testdata/isobin_manifests/default_load.toml").into()),
            ),
        )]
    #[case(
        ManifestFileExtensions::Json,
        "testdata/isobin_manifests/default_load.toml",
            SerdeExtError::new_deserialize_with_hint(
                anyhow!("expected value at line 1 column 2"),
                with_current_source_dir("testdata/isobin_manifests/default_load.toml"),
                ErrorHint::new(1,2,include_str!("testdata/isobin_manifests/default_load.toml").into()),
            ),
        )]
    #[tokio::test]
    async fn isobin_manifest_from_str_error_works(
        #[case] ft: ManifestFileExtensions,
        #[case] path: impl AsRef<Path>,
        #[case] expected: SerdeExtError,
    ) {
        let path = current_source_dir!().join(path);
        let result = IsobinManifest::parse(ft, path).await;
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
    fn tool_manifest(
        cargo_install_dependencies: Vec<(String, CargoInstallDependency)>,
    ) -> IsobinManifest {
        IsobinManifest {
            cargo: CargoManifest::new(cargo_install_dependencies.into_iter().collect()),
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
    #[case("foo.yaml", ManifestFileExtensions::Yaml)]
    #[case("foo.yml", ManifestFileExtensions::Yaml)]
    #[case("foo.toml", ManifestFileExtensions::Toml)]
    fn get_manifest_file_extension_works(
        #[case] path: &str,
        #[case] expected: ManifestFileExtensions,
    ) {
        let actual = IsobinManifest::get_file_extension(path).unwrap();
        pretty_assertions::assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("foo.fm", IsobinManifestError::new_unknown_file_extension("foo.fm".into(), "fm".into()))]
    #[case("foo", IsobinManifestError::new_nothing_file_extension("foo".into()))]
    fn get_manifest_file_extension_error_works(
        #[case] path: &str,
        #[case] expected: IsobinManifestError,
    ) {
        let result = IsobinManifest::get_file_extension(path);
        assert_error_result!(expected, result);
    }
}
