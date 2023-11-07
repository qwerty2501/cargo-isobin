use clap::{Parser, Subcommand};
use isobin::{fronts::print_error, *};
use std::{path::PathBuf, process::exit};

#[tokio::main]
async fn main() {
    let app = Application::default();

    let args = Arguments::parse();
    let result = app.run(args).await;
    match result {
        Ok(()) => {}
        Err(err) => {
            print_error(&err);
            exit(1);
        }
    }
}

#[derive(Default)]
pub struct Application {
    install_service: InstallService,
    path_service: PathService,
}

impl Application {
    pub async fn run(&self, args: Arguments) -> Result<()> {
        let service_option_builder = ServiceOptionBuilder::default();
        let service_option_builder = if let Some(isobin_config_path) = args.isobin_config_path {
            service_option_builder.isobin_config_path(isobin_config_path)
        } else {
            service_option_builder
        };
        let service_option = service_option_builder.try_build().await?;

        let subcommand = args.subcommand;
        match subcommand {
            SubCommands::Install {
                force,
                install_targets,
            } => {
                self.run_install(service_option, force, install_targets)
                    .await
            }
            SubCommands::Path => self.run_path(service_option).await,
        }
    }
    async fn run_install(
        &self,
        service_option: ServiceOption,
        force: bool,
        install_targets: Vec<String>,
    ) -> Result<()> {
        eprintln!("Start instllations.");
        let install_service_option = InstallServiceOptionBuilder::default()
            .mode(if install_targets.is_empty() {
                InstallMode::All
            } else {
                InstallMode::SpecificInstallTargetsOnly {
                    specific_install_targets: install_targets,
                }
            })
            .force(force)
            .build();
        self.install_service
            .install(service_option, install_service_option)
            .await?;
        eprintln!("Completed instllations.");
        Ok(())
    }
    async fn run_path(&self, service_option: ServiceOption) -> Result<()> {
        let path = self.path_service.path(service_option).await?;
        println!("{}", path.display());
        Ok(())
    }
}

#[derive(Parser)]
#[command(author, version, about)]
pub struct Arguments {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE", name = "config")]
    isobin_config_path: Option<PathBuf>,
    #[command(subcommand)]
    subcommand: SubCommands,
}

#[derive(Subcommand)]
pub enum SubCommands {
    Install {
        #[arg(short, long)]
        force: bool,
        install_targets: Vec<String>,
    },
    Path,
}
