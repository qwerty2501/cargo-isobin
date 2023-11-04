#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate derive_getters;

mod config;
mod errors;
pub mod fronts;
mod install;
mod macros;
mod path;
mod paths;
pub mod providers;
mod result;
mod service_option;
mod utils;
pub use errors::*;
pub use install::*;
pub use path::*;
pub use result::*;
pub use service_option::*;

use async_trait::async_trait;
pub use config::*;
#[cfg(test)]
use rstest::*;
