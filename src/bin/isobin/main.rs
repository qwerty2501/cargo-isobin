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
        let subcommand = args.subcommand;
        match subcommand {
            SubCommands::Install {
                force,
                install_targets,
            } => {
                self.run_install(args.isobin_config_path, force, install_targets)
                    .await
            }
            SubCommands::Path => self.run_path(args.isobin_config_path).await,
        }
    }
    async fn run_install(
        &self,
        isobin_config_path: Option<PathBuf>,
        force: bool,
        install_targets: Option<Vec<String>>,
    ) -> Result<()> {
        eprintln!("Start instllations.");
        let install_service_option = InstallServiceOptionBuilder::default()
            .mode(if let Some(install_targets) = install_targets {
                InstallMode::SpecificInstallTargetsOnly {
                    specific_install_targets: install_targets,
                }
            } else {
                InstallMode::All
            })
            .isobin_config_path(isobin_config_path)
            .force(force)
            .try_build()
            .await?;
        self.install_service.install(install_service_option).await?;
        eprintln!("Completed instllations.");
        Ok(())
    }
    async fn run_path(&self, isobin_config_path: Option<PathBuf>) -> Result<()> {
        let path_service_option = PathServiceOptionBuilder::default()
            .isobin_config_path(isobin_config_path)
            .try_build()
            .await?;
        let path = self.path_service.path(path_service_option).await?;
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
        #[arg(short, long, default_value_t = false)]
        force: bool,
        install_targets: Option<Vec<String>>,
    },
    Path,
}
