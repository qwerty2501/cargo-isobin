#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate derive_getters;

mod bin_map;
mod clearn;
mod errors;
mod fronts;
mod install;
mod macros;
mod manifest;
mod path;
mod paths;
mod providers;
mod result;
mod run;
mod specified_target;
mod sync;
mod utils;
pub use errors::*;
pub use fronts::print_error;
use install::InstallService;
pub use install::{InstallMode, InstallServiceOption, InstallServiceOptionBuilder};
use path::PathService;
pub use path::{PathServiceOption, PathServiceOptionBuilder};
pub use result::*;
use run::*;
pub use run::{RunServiceOption, RunServiceOptionBuilder};
use sync::SyncService;
pub use sync::{SyncServiceOption, SyncServiceOptionBuilder};

use async_trait::async_trait;
use clearn::*;
pub use clearn::{CleanServiceOption, CleanServiceOptionBuilder};
use manifest::*;
pub use providers::ProviderKind;
#[cfg(test)]
use rstest::*;
pub use specified_target::*;

pub async fn install(install_service_option: InstallServiceOption) -> Result<()> {
    let install_service = InstallService::default();
    let quiet = *install_service_option.quiet();
    flex_eprintln!(quiet, "Start instllations.");
    install_service.install(install_service_option).await?;
    flex_eprintln!(quiet, "Completed instllations.");
    Ok(())
}

pub async fn path(path_service_option: PathServiceOption) -> Result<std::path::PathBuf> {
    let path_service = PathService::default();
    path_service.path(path_service_option).await
}

pub async fn sync(sync_service_option: SyncServiceOption) -> Result<()> {
    let sync_service = SyncService::default();
    let quiet = *sync_service_option.quiet();
    flex_eprintln!(quiet, "Start sync.");
    sync_service.sync(sync_service_option).await?;
    flex_eprintln!(quiet, "Completed sync.");
    Ok(())
}
pub async fn clear(clean_service_option: CleanServiceOption) -> Result<()> {
    let clean_service = CleanService::default();
    let quiet = *clean_service_option.quiet();
    flex_eprintln!(quiet, "Start clear.");
    clean_service.clean(clean_service_option).await?;
    flex_eprintln!(quiet, "Completed clear.");
    Ok(())
}

pub async fn run(run_service_option: RunServiceOption) -> Result<()> {
    let run_service = RunService::default();
    run_service.run(run_service_option).await
}
