use super::*;
#[derive(thiserror::Error, Debug, new)]
pub enum Error {
    #[error("{0}")]
    IsobinConfig(#[from] config::IsobinConfigError),
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
pub mod test_util {

    #[macro_export]
    macro_rules! assert_error_result {
        ($expected:expr,$result:expr) => {
            if let Err(err) = $result {
                use std::any::Any;
                std::assert_eq!($expected.type_id(), err.type_id());
                pretty_assertions::assert_eq!(format!("{:?}", $expected), format!("{:?}", err));
            } else {
                panic!("unexpected result ok");
            }
        };
    }
}
