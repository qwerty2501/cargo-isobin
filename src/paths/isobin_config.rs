use super::*;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

pub async fn search_isobin_config_path(current_dir: impl AsRef<Path>) -> Result<PathBuf> {
    let mut current_dir = current_dir.as_ref();
    // TODO: Currently, rust can not call recursively async function.
    // Fix this loop when rust can call recursively async function in the future,
    loop {
        let dir_str = current_dir.to_str().unwrap_or("");
        if dir_str.is_empty() {
            return Err(IsobinConfigPathError::NotFoundIsobinConfig.into());
        }
        let target_isobin_paths = make_isobin_config_paths(current_dir);
        let mut isobin_path_futures = vec![];
        for isobin_path in target_isobin_paths.iter() {
            isobin_path_futures.push((isobin_path, isobin_path.exists()));
        }
        let mut exists_isobin_paths = vec![];
        for ipf in isobin_path_futures.into_iter() {
            if ipf.1 {
                exists_isobin_paths.push(ipf.0);
            }
        }
        match exists_isobin_paths.len().cmp(&1) {
            Ordering::Equal => return Ok(exists_isobin_paths[0].clone()),
            Ordering::Greater => {
                return Err(IsobinConfigPathError::Conflict(
                    exists_isobin_paths
                        .iter()
                        .map(|ei| ei.to_str().unwrap_or("").to_string())
                        .collect(),
                )
                .into())
            }
            Ordering::Less => {
                if let Some(parent_dir) = current_dir.parent() {
                    current_dir = parent_dir
                } else {
                    return Err(IsobinConfigPathError::NotFoundIsobinConfig.into());
                }
            }
        }
    }
}

fn make_isobin_config_paths(dir: impl AsRef<Path>) -> Vec<PathBuf> {
    let allow_isobin_config_base_names = ["isobin", ".isobin"];
    let allow_isobin_config_extensions = ["toml", "yaml", "yml", "json"];
    let mut search_isobin_config_paths = vec![];
    for base_name in allow_isobin_config_base_names.into_iter() {
        for extension in allow_isobin_config_extensions.into_iter() {
            let isobin_config_path = dir.as_ref().join(format!("{}.{}", base_name, extension));
            search_isobin_config_paths.push(isobin_config_path);
        }
    }
    search_isobin_config_paths
}

#[derive(thiserror::Error, Debug, new)]
pub enum IsobinConfigPathError {
    #[error("Conflict config files:{0:?}")]
    Conflict(Vec<String>),
    #[error("Not found isobin config file")]
    NotFoundIsobinConfig,
}
