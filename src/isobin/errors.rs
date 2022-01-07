pub use anyhow::Result;

#[derive(thiserror::Error, Debug)]
pub enum Errors {
    #[error("test error")]
    Test(),
}
