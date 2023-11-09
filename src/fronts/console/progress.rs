use std::time::Duration;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    providers::{InstallTarget, ProviderKind},
    Result,
};

#[derive(Clone)]
pub struct Progress {
    progress_bar: ProgressBar,
    provider_kind: ProviderKind,
    name: String,
}

impl Progress {
    pub fn new(progress_bar: ProgressBar, install_target: &impl InstallTarget) -> Self {
        Self {
            progress_bar,
            provider_kind: install_target.provider_kind(),
            name: install_target.name().into(),
        }
    }

    pub fn prepare_install(&self) -> Result<()> {
        self.progress_bar
            .set_style(ProgressStyle::with_template("{spinner} {prefix} {msg}")?);
        self.progress_bar
            .set_prefix(format!("{}/{}", self.provider_kind, self.name));
        self.progress_bar.set_message("waiting");
        Ok(())
    }

    pub fn already_installed(&self) -> Result<()> {
        self.progress_bar
            .set_message("already installed".green().to_string());
        self.progress_bar.finish();
        Ok(())
    }

    pub fn start_uninstall(&self) -> Result<()> {
        self.progress_bar
            .enable_steady_tick(Duration::from_millis(100));
        self.progress_bar.set_message("uninstalling");
        Ok(())
    }

    pub fn start_install(&self) -> Result<()> {
        self.progress_bar
            .enable_steady_tick(Duration::from_millis(100));
        self.progress_bar.set_message("installing");
        Ok(())
    }

    pub fn done_install(&self) -> Result<()> {
        self.progress_bar.disable_steady_tick();
        self.progress_bar
            .set_style(ProgressStyle::with_template("  {prefix} {msg}")?);
        self.progress_bar
            .set_message("done install".green().to_string());
        self.progress_bar.finish();
        Ok(())
    }

    pub fn done_uninstall(&self) -> Result<()> {
        self.progress_bar.disable_steady_tick();
        self.progress_bar
            .set_style(ProgressStyle::with_template("  {prefix} {msg}")?);
        self.progress_bar
            .set_message("done uninstall".green().to_string());
        self.progress_bar.finish();
        Ok(())
    }

    pub fn failed_install(&self) -> Result<()> {
        self.progress_bar.disable_steady_tick();
        self.progress_bar
            .set_style(ProgressStyle::with_template("  {prefix} {msg}")?);
        self.progress_bar
            .set_message("failed install".red().to_string());
        self.progress_bar.finish();
        Ok(())
    }

    pub fn failed_uninstall(&self) -> Result<()> {
        self.progress_bar.disable_steady_tick();
        self.progress_bar
            .set_style(ProgressStyle::with_template("  {prefix} {msg}")?);
        self.progress_bar
            .set_message("failed uninstall".red().to_string());
        self.progress_bar.finish();
        Ok(())
    }
}
