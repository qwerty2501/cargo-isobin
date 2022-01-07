use isobin::isobin::*;

use clap::Parser;

fn main() -> Result<()> {
    let app = Application::default();

    let args = Arguments::parse();
    app.run(args)
}
