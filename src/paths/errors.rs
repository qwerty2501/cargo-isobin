use crate::utils::serde_ext::SerdeExtError;

#[derive(thiserror::Error, Debug, new)]
pub enum PathsError {
    #[error("{0}")]
    Serde(#[from] SerdeExtError),
}

pub type Result<T> = std::result::Result<T, PathsError>;
