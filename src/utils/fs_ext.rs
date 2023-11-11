use super::*;
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use tokio::{
    fs,
    fs::{copy, File, ReadDir},
};

pub async fn create_dir_if_not_exists(dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    if !dir.exists() {
        fs::create_dir_all(&dir).await?;
    }
    Ok(())
}

pub async fn open_write_file_create_if_not_exists(file_path: impl AsRef<Path>) -> Result<File> {
    let file_path = file_path.as_ref();
    if let Some(dir) = file_path.parent() {
        create_dir_if_not_exists(dir).await?;
    }
    if !file_path.exists() {
        Ok(File::create(file_path).await?)
    } else {
        Ok(fs::OpenOptions::new().write(true).open(file_path).await?)
    }
}

pub async fn enumerate_executable_files(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let dir = dir.as_ref();
    if dir.is_dir() {
        let mut dir = read_dir(dir).await?;
        let mut paths = vec![];
        while let Some(res) = dir.next_entry().await? {
            let entry = res;
            let path = entry.path();

            if is_executable::is_executable(&path) {
                paths.push(path);
            }
        }
        Ok(paths)
    } else {
        Ok(vec![])
    }
}

pub async fn make_hard_links_in_dir(
    from_dir: impl AsRef<Path>,
    to_dir: impl AsRef<Path>,
) -> Result<()> {
    let from_dir = from_dir.as_ref();
    let to_dir = to_dir.as_ref();
    create_dir_if_not_exists(to_dir).await?;
    if from_dir.is_dir() && to_dir.is_dir() {
        let executable_files = enumerate_executable_files(from_dir).await?;
        for executable_file in executable_files.iter() {
            let file_name = executable_file.file_name().unwrap().to_str().unwrap();
            let to_file_path = to_dir.join(file_name);
            if to_file_path.exists() {
                fs::remove_file(&to_file_path).await?;
            }
            fs::hard_link(executable_file, to_file_path).await?;
        }
        Ok(())
    } else {
        Ok(())
    }
}

pub async fn clean_dir(dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    create_dir_if_not_exists(dir).await?;
    fs::remove_dir_all(dir).await?;
    create_dir_if_not_exists(dir).await?;
    Ok(())
}

pub async fn read_dir(path: impl AsRef<Path>) -> Result<ReadDir> {
    match fs::read_dir(path.as_ref()).await {
        Ok(rd) => Ok(rd),
        Err(err) => Err(anyhow!(
            "An error occurred \"{}\" in reading {}.",
            err,
            path.as_ref().display()
        )),
    }
}

#[async_recursion::async_recursion]
pub async fn copy_dir(
    from_dir: impl AsRef<Path> + Send + Sync + 'static,
    to_dir: impl AsRef<Path> + Send + Sync + 'static,
) -> Result<()> {
    create_dir_if_not_exists(&to_dir).await?;
    let mut rd = read_dir(from_dir.as_ref()).await?;
    while let Some(entry) = rd.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            let new_to_dir = to_dir
                .as_ref()
                .join(path.file_name().unwrap().to_string_lossy().as_ref());
            copy_dir(path, new_to_dir).await?;
        } else {
            copy(
                &path,
                to_dir
                    .as_ref()
                    .join(path.file_name().unwrap().to_string_lossy().as_ref()),
            )
            .await?;
        }
    }
    Ok(())
}
