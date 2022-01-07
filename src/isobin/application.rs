use super::*;

#[derive(Default)]
pub struct Application;

impl Application {
    pub fn run(&self, _args: Arguments) -> Result<()> {
        Err(Error::new_test())
    }
}
