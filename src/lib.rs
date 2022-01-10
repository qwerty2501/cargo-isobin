#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate derive_getters;

mod config;
mod installer;
mod macros;
mod paths;
pub mod providers;
mod utils;
pub use installer::*;

use async_trait::async_trait;
pub use config::*;
#[cfg(test)]
use rstest::*;
