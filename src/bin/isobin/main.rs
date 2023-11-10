use clap::{Parser, Subcommand};
use isobin::{print_error, *};
use std::{path::PathBuf, process::exit};

#[tokio::main]
async fn main() {
    let app = Application;

    let args = Arguments::parse();
    let result = app.exec(args).await;
    match result {
        Ok(()) => {}
        Err(err) => {
            print_error(&err);
            exit(1);
        }
    }
}

pub struct Application;

impl Application {
    pub async fn exec(&self, args: Arguments) -> Result<()> {
        let subcommand = args.subcommand;
        match subcommand {
            SubCommands::Install {
                force,
                install_targets,
            } => {
                self.install(args.isobin_config_path, force, install_targets)
                    .await
            }
            SubCommands::Path => self.path(args.isobin_config_path).await,
        }
    }
    async fn install(
        &self,
        isobin_config_path: Option<PathBuf>,
        force: bool,
        install_targets: Option<Vec<String>>,
    ) -> Result<()> {
        eprintln!("Start instllations.");
        let install_service_option_builder = InstallServiceOptionBuilder::default()
            .mode(if let Some(install_targets) = install_targets {
                InstallMode::SpecificInstallTargetsOnly {
                    specific_install_targets: install_targets,
                }
            } else {
                InstallMode::All
            })
            .force(force);
        let install_service_option_builder = if let Some(isobin_config_path) = isobin_config_path {
            install_service_option_builder.isobin_config_path(isobin_config_path)
        } else {
            install_service_option_builder
        };

        install(install_service_option_builder.build()).await?;
        eprintln!("Completed instllations.");
        Ok(())
    }
    async fn path(&self, isobin_config_path: Option<PathBuf>) -> Result<()> {
        let path_service_option_builder = PathServiceOptionBuilder::default();
        let path_service_option_builder = if let Some(isobin_config_path) = isobin_config_path {
            path_service_option_builder.isobin_config_path(isobin_config_path)
        } else {
            path_service_option_builder
        };
        let path = path(path_service_option_builder.build()).await?;
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
