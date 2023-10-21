#[allow(unused_imports)]
use super::*;
use strum_macros::{Display, IntoStaticStr};

pub mod cargo;
mod installer;

pub use installer::*;

#[derive(PartialEq, Debug, IntoStaticStr, Display)]
pub enum ProviderKind {
    #[strum(serialize = "cargo")]
    Cargo,
}
