#[allow(unused_imports)]
use super::*;

pub mod cargo;
mod installer;

pub use installer::*;

#[derive(PartialEq, Debug)]
pub enum ProviderKind {
    Cargo,
}
