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
                self.install(args.manifest_path, args.quiet, force, install_targets)
                    .await
            }
            SubCommands::Path => self.path(args.manifest_path, args.quiet).await,
        }
    }
    async fn install(
        &self,
        isobin_manifest_path: Option<PathBuf>,
        quiet: bool,
        force: bool,
        install_targets: Option<Vec<String>>,
    ) -> Result<()> {
        let install_service_option_builder = InstallServiceOptionBuilder::default()
            .quiet(quiet)
            .mode(if let Some(install_targets) = install_targets {
                InstallMode::SpecificInstallTargetsOnly {
                    specific_install_targets: install_targets,
                }
            } else {
                InstallMode::All
            })
            .force(force);
        let install_service_option_builder =
            if let Some(isobin_manifest_path) = isobin_manifest_path {
                install_service_option_builder.isobin_manifest_path(isobin_manifest_path)
            } else {
                install_service_option_builder
            };

        install(install_service_option_builder.build()).await?;
        Ok(())
    }
    async fn path(&self, isobin_manifest_path: Option<PathBuf>, quiet: bool) -> Result<()> {
        let path_service_option_builder = PathServiceOptionBuilder::default().quiet(quiet);
        let path_service_option_builder = if let Some(isobin_manifest_path) = isobin_manifest_path {
            path_service_option_builder.isobin_manifest_path(isobin_manifest_path)
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
    /// Sets a custom manifest file
    #[arg(long, value_name = "PATH")]
    manifest_path: Option<PathBuf>,
    #[arg(long, short, default_value_t = false)]
    quiet: bool,
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
