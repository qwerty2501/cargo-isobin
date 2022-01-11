use super::*;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    ServiceOptionBuild(#[from] ServiceOptionBuildError),
    #[error("{0}")]
    InstallService(#[from] InstallServiceError),
}
pub type Result<T> = std::result::Result<T, Error>;
