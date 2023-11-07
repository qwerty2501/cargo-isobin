use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde_derive::{Deserialize, Serialize};

use crate::Result;

use super::{fs_ext, join_future::join_all};

#[derive(Deserialize, Serialize, PartialEq)]
pub struct FileModifiedCache {
    size: u64,
    modifieded_at: Option<SystemTime>,
}

pub struct FileModifiedCacheSet {
    path: PathBuf,
    cache: FileModifiedCache,
}

#[derive(Deserialize, Serialize)]
pub struct FileModifiedCacheMap {
    files: HashMap<PathBuf, FileModifiedCache>,
}

pub const FILE_MODIFIED_CACHE_MAP_FILE_NAME: &str = "file_modifid_cache.v1.json";

pub async fn has_file_diff_in_dir(
    dir: impl AsRef<Path>,
    target_exts: Vec<String>,
    target_file_names: Vec<String>,
    exclude_names: Vec<String>,
    modified_cache_map: FileModifiedCacheMap,
) -> Result<bool> {
    let target_ext_map = target_exts.into_iter().collect::<HashSet<_>>();
    let target_file_name_map = target_file_names.into_iter().collect::<HashSet<_>>();
    let exclude_name_map = exclude_names.into_iter().collect::<HashSet<_>>();
    let target_files = enumurate_target_files(
        dir.as_ref(),
        &target_ext_map,
        &target_file_name_map,
        &exclude_name_map,
    )
    .await?;
    let cache_sets = join_all(target_files.into_iter().map(get_file_modified_cache)).await?;
    for cache in cache_sets.iter() {
        if let Some(old_cache) = modified_cache_map.files.get(&cache.path) {
            if &cache.cache != old_cache {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

async fn get_file_modified_cache(path: PathBuf) -> Result<FileModifiedCacheSet> {
    let file = tokio::fs::File::open(&path).await?;
    let metadata = file.metadata().await?;
    let modifieded_at = if let Ok(modifieded_at) = metadata.modified() {
        Some(modifieded_at)
    } else {
        None
    };
    let size = metadata.len();
    Ok(FileModifiedCacheSet {
        path,
        cache: FileModifiedCache {
            size,
            modifieded_at,
        },
    })
}

#[async_recursion::async_recursion]
pub async fn enumurate_target_files(
    dir: impl AsRef<Path> + std::marker::Send + std::marker::Sync + 'async_recursion,
    target_ext_map: &HashSet<String>,
    target_file_name_map: &HashSet<String>,
    exclude_name_map: &HashSet<String>,
) -> Result<Vec<PathBuf>> {
    let mut read_dir = fs_ext::read_dir(dir).await?;
    let mut paths = vec![];
    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            let file_name = file_name.to_string_lossy();
            if exclude_name_map.get(file_name.as_ref()).is_some() {
                continue;
            }
        }
        if path.is_dir() {
            paths.extend_from_slice(
                &enumurate_target_files(
                    path,
                    target_ext_map,
                    target_file_name_map,
                    exclude_name_map,
                )
                .await?,
            );
        } else if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy();
            if target_ext_map.get(ext.as_ref()).is_some() {
                paths.push(path)
            }
        } else if let Some(file_name) = path.file_name() {
            let file_name = file_name.to_string_lossy();
            if target_file_name_map.get(file_name.as_ref()).is_some() {
                paths.push(path)
            }
        }
    }
    Ok(paths)
}
