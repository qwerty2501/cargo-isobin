use tokio::fs;

use super::*;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

pub fn isobin_manifest_dir(isobin_manifest_path: &Path) -> Result<&Path> {
    let isobin_manifest_dir = isobin_manifest_path
        .parent()
        .ok_or_else(IsobinManifestPathError::new_not_found_isobin_manifest)?;
    Ok(isobin_manifest_dir)
}

pub async fn isobin_manifest_path_canonicalize(
    isobin_manifest_path: Option<PathBuf>,
) -> Result<PathBuf> {
    let isobin_manifest_path = if let Some(isobin_manifest_path) = isobin_manifest_path {
        isobin_manifest_path
    } else {
        let current_dir = std::env::current_dir().unwrap();
        search_isobin_manifest_path(current_dir).await?
    };
    Ok(fs::canonicalize(isobin_manifest_path).await?)
}

pub async fn search_isobin_manifest_path(current_dir: impl AsRef<Path>) -> Result<PathBuf> {
    let mut current_dir = current_dir.as_ref();
    // TODO: Currently, rust can not call recursively async function.
    // Fix this loop when rust can call recursively async function in the future,
    loop {
        let dir_str = current_dir.to_str().unwrap_or("");
        if dir_str.is_empty() {
            return Err(IsobinManifestPathError::NotFoundIsobinManifest.into());
        }
        let target_isobin_paths = make_isobin_manifest_paths(current_dir);
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
                return Err(IsobinManifestPathError::Conflict(
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
                    return Err(IsobinManifestPathError::NotFoundIsobinManifest.into());
                }
            }
        }
    }
}

fn make_isobin_manifest_paths(dir: impl AsRef<Path>) -> Vec<PathBuf> {
    let base_name = "isobin";
    let allow_isobin_manifest_extensions = ["toml", "yaml", "yml", "json"];
    let mut search_isobin_manifest_paths = vec![];
    for extension in allow_isobin_manifest_extensions.into_iter() {
        let isobin_manifest_path = dir.as_ref().join(format!("{}.{}", base_name, extension));
        search_isobin_manifest_paths.push(isobin_manifest_path);
    }
    search_isobin_manifest_paths
}

#[derive(thiserror::Error, Debug, new)]
pub enum IsobinManifestPathError {
    #[error("Conflict config files:{0:?}")]
    Conflict(Vec<String>),
    #[error("Not found isobin config file")]
    NotFoundIsobinManifest,
}
