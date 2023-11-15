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
                cargo_install_targets,
                install_targets,
            } => {
                self.install(
                    args.manifest_path,
                    args.quiet,
                    force,
                    cargo_install_targets,
                    install_targets,
                )
                .await
            }
            SubCommands::Path => self.path(args.manifest_path, args.quiet).await,
            SubCommands::Sync { force } => self.sync(args.manifest_path, args.quiet, force).await,
            SubCommands::Clean => self.clean(args.manifest_path, args.quiet).await,
            SubCommands::Run {
                quiet,
                bin,
                arguments,
            } => self.run(args.manifest_path, quiet, bin, arguments).await,
        }
    }
    async fn install(
        &self,
        isobin_manifest_path: Option<PathBuf>,
        quiet: bool,
        force: bool,
        cargo_install_targets: Option<Vec<String>>,
        install_targets: Option<Vec<String>>,
    ) -> Result<()> {
        let mut targets = vec![];
        if let Some(cargo_install_targets) = cargo_install_targets {
            for target in cargo_install_targets.into_iter() {
                targets.push(SpecifiedTarget::new(Some(ProviderKind::Cargo), target));
            }
        }
        if let Some(install_targets) = install_targets {
            for target in install_targets.into_iter() {
                targets.push(SpecifiedTarget::new(None, target));
            }
        }

        let install_service_option_builder = InstallServiceOptionBuilder::default()
            .quiet(quiet)
            .mode(if !targets.is_empty() {
                InstallMode::SpecificInstallTargetsOnly {
                    specified_install_targets: targets,
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

        install(install_service_option_builder.build()).await
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

    async fn sync(
        &self,
        isobin_manifest_path: Option<PathBuf>,
        quiet: bool,
        force: bool,
    ) -> Result<()> {
        let sync_service_option_builder = SyncServiceOptionBuilder::default()
            .quiet(quiet)
            .force(force);
        let sync_service_option_builder = if let Some(isobin_manifest_path) = isobin_manifest_path {
            sync_service_option_builder.isobin_manifest_path(isobin_manifest_path)
        } else {
            sync_service_option_builder
        };
        sync(sync_service_option_builder.build()).await
    }
    async fn clean(&self, isobin_manifest_path: Option<PathBuf>, quiet: bool) -> Result<()> {
        let clean_service_option_builder = CleanServiceOptionBuilder::default().quiet(quiet);
        let clean_service_option_builder = if let Some(isobin_manifest_path) = isobin_manifest_path
        {
            clean_service_option_builder.isobin_manifest_path(isobin_manifest_path)
        } else {
            clean_service_option_builder
        };
        clear(clean_service_option_builder.build()).await
    }
    async fn run(
        &self,
        isobin_manifest_path: Option<PathBuf>,
        quiet: bool,
        bin: String,
        arguments: Option<Vec<String>>,
    ) -> Result<()> {
        let run_service_option_builder = RunServiceOptionBuilder::default().quiet(quiet).bin(bin);
        let run_service_option_builder = if let Some(isobin_manifest_path) = isobin_manifest_path {
            run_service_option_builder.isobin_manifest_path(isobin_manifest_path)
        } else {
            run_service_option_builder
        };
        let run_service_option_builder = if let Some(arguments) = arguments {
            run_service_option_builder.args(arguments)
        } else {
            run_service_option_builder
        };
        run(run_service_option_builder.build()).await
    }
}

#[derive(Parser)]
#[command(author, version, about)]
pub struct Arguments {
    /// Sets a custom manifest file
    #[arg(long, value_name = "PATH",value_hint = clap::ValueHint::FilePath)]
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
        #[arg(long = "cargo")]
        cargo_install_targets: Option<Vec<String>>,
        install_targets: Option<Vec<String>>,
    },
    Path,
    Sync {
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
    Clean,
    Run {
        #[arg(long, short, default_value_t = false)]
        quiet: bool,
        bin: String,
        arguments: Option<Vec<String>>,
    },
}
