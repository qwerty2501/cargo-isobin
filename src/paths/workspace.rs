use super::*;
use async_std::{
    fs::File,
    io::ReadExt,
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
pub async fn unique_workspace_dir(isobin_config_dir: impl AsRef<Path>) -> Result<PathBuf> {
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
        id
    };
    let unique_workspace_dir = workspace_dir().join(id);
    create_dir_if_not_exists(&unique_workspace_dir).await?;
    Ok(unique_workspace_dir)
}

#[derive(Deserialize, Serialize, Default, Debug)]
struct WorkspacePathMap {
    #[serde(default)]
    workspace_path_map: HashMap<String, String>,
}

impl WorkspacePathMap {
    async fn load() -> Result<WorkspacePathMap> {
        let config_dir = projects::config_dir();
        create_dir_if_not_exists(&config_dir).await?;

        const WORKSPACE_PATH_MAP_FILE_NAME: &str = "workspace_map.json";
        let workspace_path_map_file_path = config_dir.join(WORKSPACE_PATH_MAP_FILE_NAME);
        create_file_if_not_exists(&workspace_path_map_file_path).await?;
        match File::open(&workspace_path_map_file_path).await {
            Ok(mut file) => {
                let mut content = String::new();
                match file.read_to_string(&mut content).await {
                    Ok(_) => Self::load_str(&content, workspace_path_map_file_path),
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

    fn load_str(s: &str, path: impl AsRef<Path>) -> Result<WorkspacePathMap> {
        serde_json::from_str(s)
            .map_err(|e| PathsError::new_parse_workspace_map(path.as_ref().into(), e.into()))
    }
}
