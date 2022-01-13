use super::errors;

pub type Result<T> = std::result::Result<T, errors::Error>;
