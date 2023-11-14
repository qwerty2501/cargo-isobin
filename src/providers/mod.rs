#[allow(unused_imports)]
use super::*;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, IntoStaticStr};

pub mod cargo;
mod installer;

pub use installer::*;

#[derive(PartialEq, Debug, Clone, IntoStaticStr, Display, Deserialize, Serialize)]
pub enum ProviderKind {
    #[serde(rename = "cargo")]
    #[strum(serialize = "cargo")]
    Cargo,
}
