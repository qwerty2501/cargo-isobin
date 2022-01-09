#[cfg(test)]
pub mod test_util {
    #[macro_export]
    macro_rules! current_source_dir {
        () => {{
            const FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/", file!());
            Path::new(FILE).parent().unwrap()
        }};
    }
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
