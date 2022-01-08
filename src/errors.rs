#[derive(thiserror::Error, Debug, new)]
pub enum Error {
    #[error("{0}")]
    IsobinConfig(#[from] IsobinConfigError),
}

#[derive(thiserror::Error, Debug, new)]
pub enum IsobinConfigError {
    #[error("Failed read isobin install config")]
    ReadIsobinConfig(anyhow::Error),

    #[error("Failed parse isobin config:{0}")]
    ParseIsobinConfig(anyhow::Error),

    #[error("The target file does not have extension")]
    NothingFileExtension,

    #[error("The target file has unknown extension:{0}")]
    UnknownFileExtension(String),
}

pub type Result<T> = std::result::Result<T, Error>;
