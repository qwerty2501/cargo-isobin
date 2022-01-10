use async_std::path::Path;

pub fn path_to_string(path: impl AsRef<Path>) -> String {
    path.as_ref().to_str().unwrap_or("").to_string()
}
