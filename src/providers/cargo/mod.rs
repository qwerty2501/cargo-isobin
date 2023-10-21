mod config;
mod home;
mod installer;

#[allow(unused_imports)]
use super::*;
pub use config::*;
use home::*;
pub use installer::*;

pub const PROVIDER_NAME: &str = "cargo";
