#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate derive_getters;

mod config;
mod errors;
mod fronts;
mod install;
mod macros;
mod path;
mod paths;
mod providers;
mod result;
mod utils;
pub use errors::*;
pub use fronts::print_error;
use install::*;
pub use install::{InstallMode, InstallService, InstallServiceOption, InstallServiceOptionBuilder};
pub use path::{PathService, PathServiceOption, PathServiceOptionBuilder};
pub use result::*;

use async_trait::async_trait;
use config::*;
#[cfg(test)]
use rstest::*;

pub async fn install(install_service_option: InstallServiceOption) -> Result<()> {
    let install_service = InstallService::default();
    install_service.install(install_service_option).await
}

pub async fn path(path_service_option: PathServiceOption) -> Result<std::path::PathBuf> {
    let path_service = PathService::default();
    path_service.path(path_service_option).await
}
