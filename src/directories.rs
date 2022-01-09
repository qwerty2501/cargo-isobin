#[cfg(test)]
pub mod test_util {
    #[macro_export]
    macro_rules! current_source_dir {
        () => {{
            const FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/", file!());
            Path::new(FILE).parent().unwrap()
        }};
    }
}
