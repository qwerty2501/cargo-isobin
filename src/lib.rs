#[macro_use]
extern crate derive_new;

mod config;
mod directories;
mod errors;
mod installer;
pub mod isobin;
mod macros;
pub mod providers;
pub use installer::*;

use async_trait::async_trait;
pub use config::*;
#[cfg(test)]
use rstest::*;
