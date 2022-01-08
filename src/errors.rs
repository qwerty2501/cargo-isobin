#[derive(thiserror::Error, Debug, new)]
pub enum Error {
    #[error("failed read isobin install config")]
    ReadIsobinInstallConfigError(anyhow::Error),

    #[error("{0}")]
    ParseIsobinInstallConfigError(anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
