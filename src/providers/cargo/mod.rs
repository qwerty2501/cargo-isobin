mod home;
mod installer;
mod manifest;

#[allow(unused_imports)]
use super::*;
use home::*;
pub use installer::*;
pub use manifest::*;

pub const PROVIDER_NAME: &str = "cargo";
