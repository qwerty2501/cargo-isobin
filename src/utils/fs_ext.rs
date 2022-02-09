use async_std::path::{Path, PathBuf};
use async_std::prelude::*;
use async_std::{fs, fs::File};
pub async fn create_dir_if_not_exists(dir: impl AsRef<Path>) -> Result<(), async_std::io::Error> {
    let dir = dir.as_ref();
    if !dir.exists().await {
        fs::create_dir_all(&dir).await?;
    }
    Ok(())
}

pub async fn open_file_create_if_not_exists(
    file_path: impl AsRef<Path>,
) -> Result<File, async_std::io::Error> {
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

pub async fn enumerate_executable_files(
    dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, async_std::io::Error> {
    let dir = dir.as_ref();
    if dir.is_dir().await {
        let mut dir = dir.read_dir().await?;

        let mut paths = vec![];
        while let Some(res) = dir.next().await {
            let entry = res?;
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
) -> Result<(), async_std::io::Error> {
    let from_dir = from_dir.as_ref();
    let to_dir = to_dir.as_ref();
    create_dir_if_not_exists(to_dir).await?;
    if from_dir.is_dir().await && to_dir.is_dir().await {
        let executable_files = enumerate_executable_files(from_dir).await?;
        for executable_file in executable_files.iter() {
            let file_name = executable_file.file_name().unwrap().to_str().unwrap();
            let to_file_path = to_dir.join(file_name);
            fs::hard_link(executable_file, to_file_path).await?;
        }
        Ok(())
    } else {
        Ok(())
    }
}

pub async fn clean_dir(dir: impl AsRef<Path>) -> Result<(), async_std::io::Error> {
    let dir = dir.as_ref();
    create_dir_if_not_exists(dir).await?;
    fs::remove_dir_all(dir).await?;
    create_dir_if_not_exists(dir).await?;
    Ok(())
}
