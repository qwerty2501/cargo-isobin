use crate::paths::isobin_config::IsobinConfigPathError;
use crate::utils::serde_ext::SerdeExtError;
use crate::InstallServiceError;
use crate::IsobinConfigError;

#[derive(thiserror::Error, Debug, new)]
pub enum Error {
    #[error("{0}")]
    IsobinConfig(#[from] IsobinConfigError),
    #[error("{0}")]
    IsobinConfigPath(#[from] IsobinConfigPathError),
    #[error("{0}")]
    Serde(#[from] SerdeExtError),
    #[error("{0}")]
    InstallService(#[from] InstallServiceError),
}
