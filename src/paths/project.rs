use std::path::PathBuf;

#[cfg(debug_assertions)]
const PACKAGE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "_dev");
#[cfg(not(debug_assertions))]
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Getters, Clone, PartialEq, Debug, new)]
pub struct Project {
    config_dir: PathBuf,
    cache_dir: PathBuf,
    data_local_dir: PathBuf,
}

impl Default for Project {
    fn default() -> Self {
        let pds = project_dirs();
        Self {
            config_dir: pds.config_dir().into(),
            cache_dir: pds.cache_dir().into(),
            data_local_dir: pds.data_local_dir().into(),
        }
    }
}

fn project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("org", PACKAGE_NAME, PACKAGE_NAME).unwrap()
}
