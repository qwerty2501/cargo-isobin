mod application;
mod arguments;
mod errors;
pub use application::*;
pub use arguments::*;
pub use errors::*;

use clap::Parser;

fn main() -> Result<()> {
    let app = Application::default();

    let args = Arguments::parse();
    app.run(args)
}
