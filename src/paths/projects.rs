use async_std::path::PathBuf;

#[cfg(debug_assertions)]
const PACKAGE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "_dev");
#[cfg(not(debug_assertions))]
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

#[allow(dead_code)]
pub fn config_dir() -> PathBuf {
    let pd = project_dirs();
    pd.config_dir().into()
}

#[allow(dead_code)]
pub fn cache_dir() -> PathBuf {
    let pd = project_dirs();
    pd.cache_dir().into()
}

pub fn project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("org", PACKAGE_NAME, PACKAGE_NAME).unwrap()
}
