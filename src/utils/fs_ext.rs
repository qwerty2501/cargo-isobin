use async_std::path::Path;
use async_std::{fs, fs::File};

type Result<T> = std::result::Result<T, async_std::io::Error>;
pub async fn create_dir_if_not_exists(dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    if !dir.exists().await {
        fs::create_dir_all(&dir).await?;
    }
    Ok(())
}

pub async fn open_file_create_if_not_exists(file_path: impl AsRef<Path>) -> Result<File> {
    let file_path = file_path.as_ref();
    if let Some(dir) = file_path.parent() {
        create_dir_if_not_exists(dir).await?;
    }
    if !file_path.exists().await {
        File::create(file_path).await
    } else {
        File::open(file_path).await
    }
}
