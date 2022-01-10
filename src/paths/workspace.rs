use crate::utils::serde_ext::Json;

use super::*;
use async_std::path::{Path, PathBuf};
use errors::Result;
use nanoid::nanoid;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Getters, Clone, PartialEq, Debug)]
pub struct Workspace {
    base_dir: PathBuf,
    tmp_dir: PathBuf,
    home_dir: PathBuf,
}

impl Workspace {
    #[allow(dead_code)]
    fn new(base_unique_workspace_dir: PathBuf) -> Self {
        Self {
            base_dir: base_unique_workspace_dir.clone(),
            tmp_dir: base_unique_workspace_dir.join("tmp"),
            home_dir: base_unique_workspace_dir.join("home"),
        }
    }
}

#[derive(new)]
pub struct WorkspaceProvider {
    config_dir: PathBuf,
    workspace_dir: PathBuf,
}

impl WorkspaceProvider {
    #[allow(dead_code)]
    pub async fn base_unique_workspace_dir_from_isobin_config_dir(
        &self,
        isobin_config_dir: impl AsRef<Path>,
    ) -> Result<Workspace> {
        let mut workspace_path_map =
            WorkspacePathMap::parse_from_config_dir(&self.config_dir).await?;
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
            WorkspacePathMap::save_to_config_dir(&workspace_path_map, &self.config_dir).await?;
            id
        };
        let base_unique_workspace_dir = self.workspace_dir.join(id);
        Ok(Workspace::new(base_unique_workspace_dir))
    }
}

#[allow(dead_code)]
pub fn workspace_dir() -> PathBuf {
    projects::cache_dir().join("workspace")
}

#[derive(Deserialize, Serialize, Default, Debug)]
struct WorkspacePathMap {
    #[serde(default, flatten)]
    workspace_path_map: HashMap<String, String>,
}

impl WorkspacePathMap {
    const WORKSPACE_PATH_MAP_FILE_NAME: &'static str = "workspace_map.json";
    async fn parse_from_config_dir(config_dir: impl AsRef<Path>) -> Result<WorkspacePathMap> {
        let workspace_path_map_file_path =
            config_dir.as_ref().join(Self::WORKSPACE_PATH_MAP_FILE_NAME);
        Ok(Json::parse_from_file(workspace_path_map_file_path).await?)
    }

    async fn save_to_config_dir(
        workspace_path_map: &Self,
        config_dir: impl AsRef<Path>,
    ) -> Result<()> {
        let workspace_path_map_file_path =
            config_dir.as_ref().join(Self::WORKSPACE_PATH_MAP_FILE_NAME);
        Ok(Json::save_to_file(workspace_path_map, workspace_path_map_file_path).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    #[case("/home/user_name/.cache/332334".into(),Workspace{
        base_dir:"/home/user_name/.cache/332334".into(),
        tmp_dir:"/home/user_name/.cache/332334/tmp".into(),
        home_dir:"/home/user_name/.cache/332334/home".into(),
    })]
    fn workspace_new_works(
        #[case] base_unique_workspace_dir: PathBuf,
        #[case] expected: Workspace,
    ) {
        let actual = Workspace::new(base_unique_workspace_dir);
        pretty_assertions::assert_eq!(expected, actual);
    }
}
