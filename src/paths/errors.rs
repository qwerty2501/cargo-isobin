use async_std::path::PathBuf;

#[derive(thiserror::Error, Debug, new)]
pub enum PathsError {
    #[error("Failed create the dir \npath:{path:?}\nerror:{error}")]
    CreateDir {
        path: PathBuf,
        #[source]
        error: anyhow::Error,
    },
    #[error("Failed create the file \npath:{path:?}\nerror:{error}")]
    CreateFile {
        path: PathBuf,
        #[source]
        error: anyhow::Error,
    },
    #[error("Failed read the workspace path map file\nworkspace_path_map file path:{path:?}\nerror:{error}")]
    ReadWorkspaceMap {
        path: PathBuf,
        #[source]
        error: anyhow::Error,
    },

    #[error("Failed parse the workspace path map file\nworkspace_path_map file path:{path:?}\nerror:{error}")]
    ParseWorkspaceMap {
        path: PathBuf,
        #[source]
        error: anyhow::Error,
    },
}

pub type Result<T> = std::result::Result<T, PathsError>;
