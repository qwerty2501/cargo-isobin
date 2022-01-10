use super::*;
use async_std::{
    fs::File,
    io::{ReadExt, WriteExt},
    path::{Path, PathBuf},
};
use errors::{PathsError, Result};
use nanoid::nanoid;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

fn workspace_dir() -> PathBuf {
    projects::cache_dir().join("workspace")
}

#[allow(dead_code)]
pub async fn unique_isobin_workspace_dir(isobin_config_dir: impl AsRef<Path>) -> Result<PathBuf> {
    let mut workspace_path_map = WorkspacePathMap::load().await?;
    let id = if let Some(id) = workspace_path_map
        .workspace_path_map
        .get(isobin_config_dir.as_ref().to_str().unwrap())
    {
        id.into()
    } else {
        let id = nanoid!();
        workspace_path_map.workspace_path_map.insert(
            isobin_config_dir.as_ref().to_str().unwrap().into(),
            id.to_string(),
        );
        WorkspacePathMap::save(&workspace_path_map).await?;
        id
    };
    let unique_isobin_workspace_dir = workspace_dir().join(id);
    Ok(unique_isobin_workspace_dir)
}

#[derive(Deserialize, Serialize, Default, Debug)]
struct WorkspacePathMap {
    #[serde(default, flatten)]
    workspace_path_map: HashMap<String, String>,
}

impl WorkspacePathMap {
    async fn load() -> Result<WorkspacePathMap> {
        let config_dir = projects::config_dir();
        Self::from_config_dir(config_dir).await
    }

    const WORKSPACE_PATH_MAP_FILE_NAME: &'static str = "workspace_map.json";
    async fn from_config_dir(config_dir: impl AsRef<Path>) -> Result<WorkspacePathMap> {
        create_dir_if_not_exists(&config_dir).await?;

        let workspace_path_map_file_path =
            config_dir.as_ref().join(Self::WORKSPACE_PATH_MAP_FILE_NAME);
        create_file_if_not_exists(&workspace_path_map_file_path).await?;
        match File::open(&workspace_path_map_file_path).await {
            Ok(mut file) => {
                let mut content = String::new();
                match file.read_to_string(&mut content).await {
                    Ok(_) => Self::parse(&content, workspace_path_map_file_path),
                    Err(err) => Err(PathsError::new_read_workspace_map(
                        workspace_path_map_file_path,
                        err.into(),
                    )),
                }
            }
            Err(err) => {
                if err.kind() == async_std::io::ErrorKind::NotFound {
                    Ok(Default::default())
                } else {
                    Err(PathsError::new_read_workspace_map(
                        workspace_path_map_file_path,
                        err.into(),
                    ))
                }
            }
        }
    }

    fn parse(s: &str, path: impl AsRef<Path>) -> Result<WorkspacePathMap> {
        serde_json::from_str(s)
            .map_err(|e| PathsError::new_parse_workspace_map(path.as_ref().into(), e.into()))
    }

    async fn save(workspace_path_map: &Self) -> Result<()> {
        let config_dir = projects::config_dir();
        Self::save_to_config_dir(workspace_path_map, config_dir).await
    }

    async fn save_to_config_dir(
        workspace_path_map: &Self,
        config_dir: impl AsRef<Path>,
    ) -> Result<()> {
        create_dir_if_not_exists(&config_dir).await?;
        let workspace_path_map_file_path =
            config_dir.as_ref().join(Self::WORKSPACE_PATH_MAP_FILE_NAME);
        let s = Self::serialize(workspace_path_map, &workspace_path_map_file_path)?;
        create_file_if_not_exists(&workspace_path_map_file_path).await?;
        let mut file = File::open(&workspace_path_map_file_path)
            .await
            .map_err(|e| {
                PathsError::new_save_workspace_map(workspace_path_map_file_path.clone(), e.into())
            })?;
        file.write_all(s.as_bytes())
            .await
            .map_err(|e| PathsError::new_save_workspace_map(workspace_path_map_file_path, e.into()))
    }

    fn serialize(workspace_path_map: &Self, path: impl AsRef<Path>) -> Result<String> {
        serde_json::to_string(workspace_path_map)
            .map_err(|e| PathsError::new_save_workspace_map(path.as_ref().into(), e.into()))
    }
}
