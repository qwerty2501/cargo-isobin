#[allow(unused_imports)]
use super::*;
pub mod errors;
pub mod projects;
pub mod workspace;
use async_std::fs::{self, File};
use async_std::path::Path;
use errors::*;

async fn create_dir_if_not_exists(dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    if !dir.exists().await {
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| errors::PathsError::new_create_dir(dir.into(), e.into()))?;
    }
    Ok(())
}

async fn create_file_if_not_exists(file: impl AsRef<Path>) -> Result<()> {
    let file = file.as_ref();
    if !file.exists().await {
        File::create(file)
            .await
            .map_err(|e| PathsError::new_create_file(file.into(), e.into()))?;
    }
    Ok(())
}
