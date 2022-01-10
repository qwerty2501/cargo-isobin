#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate derive_getters;

mod config;
mod install;
mod macros;
mod paths;
pub mod providers;
mod utils;
pub use install::*;

use async_trait::async_trait;
pub use config::*;
#[cfg(test)]
use rstest::*;
