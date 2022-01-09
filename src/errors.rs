use super::*;
#[derive(thiserror::Error, Debug, new)]
pub enum Error {
    #[error("{0}")]
    IsobinConfig(#[from] config::IsobinConfigError),
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, Error>;
