mod errors;
use clap::{Parser, Subcommand};
pub use errors::*;
use isobin::*;
use std::path::PathBuf;

#[async_std::main]
async fn main() -> Result<()> {
    let app = Application::default();

    let args = Arguments::parse();
    app.run(args).await
}

#[derive(Default)]
pub struct Application {
    install_service: InstallService,
}

impl Application {
    pub async fn run(&self, args: Arguments) -> Result<()> {
        let service_option_builder = ServiceOptionBuilder::new();
        let service_option_builder = if let Some(isobin_config_path) = args.isobin_config_path {
            service_option_builder.isobin_config_path(isobin_config_path)
        } else {
            service_option_builder
        };
        let service_option = service_option_builder.try_build().await?;

        let subcommand = args.subcommand;
        match subcommand {
            SubCommands::Install { install_targets } => {
                let install_service_option = InstallServiceOptionBuilder::new()
                    .mode(if install_targets.is_empty() {
                        InstallMode::All
                    } else {
                        InstallMode::SpecificInstallTargetsOnly {
                            specific_install_targets: install_targets,
                        }
                    })
                    .build();
                self.install_service
                    .execute(&service_option, &install_service_option)
                    .await?;
                Ok(())
            }
        }
    }
}
#[derive(Parser)]
#[clap(author, version, about)]
pub struct Arguments {
    /// Sets a custom config file
    #[clap(short, long, parse(from_os_str), value_name = "FILE", name = "config")]
    isobin_config_path: Option<PathBuf>,
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Subcommand)]
pub enum SubCommands {
    Install { install_targets: Vec<String> },
}
