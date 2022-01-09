#[allow(unused_imports)]
use super::*;

pub mod cargo;

#[derive(PartialEq, Debug)]
pub enum ProviderType {
    Cargo,
}
