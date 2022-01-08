#[derive(thiserror::Error, Debug, new)]
pub enum Error {
    #[error("{0}")]
    IsobinConfig(#[from] IsobinConfigError),
}

#[derive(thiserror::Error, Debug, new)]
pub enum IsobinConfigError {
    #[error("Failed read isobin install config")]
    ReadIsobinConfig(#[source] anyhow::Error),

    #[error("Failed parse isobin config:{0}")]
    ParseIsobinConfig(#[source] anyhow::Error),

    #[error("The target file does not have extension")]
    NothingFileExtension,

    #[error("The target file has unknown extension:{0}")]
    UnknownFileExtension(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
pub mod test_util {
    pub fn assert_same_error<T, E: 'static + std::error::Error>(
        expected: &E,
        result: &Result<T, E>,
    ) {
        if let Err(err) = result {
            pretty_assertions::assert_eq!(format!("{:?}", expected), format!("{:?}", err));
        } else {
            panic!("unexpected result ok");
        }
    }
}
